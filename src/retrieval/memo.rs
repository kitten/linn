//! Bounded reuse of exact query encodings and full block-matrix results.
//! @ref LLP 0002#exact-numerical-reuse
use crate::{
    error::Result,
    storage::{BlockMatrix, Store},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct Memo {
    encodings: HashMap<String, Arc<Vec<f32>>>,
    key_bytes: usize,
    block_scores: Option<BlockScores>,
}
struct BlockScores {
    matrix: Arc<BlockMatrix>,
    query: Vec<u32>,
    values: Arc<Vec<f32>>,
}
impl Memo {
    pub(crate) fn clear_encodings(&mut self) {
        self.encodings.clear();
        self.key_bytes = 0;
    }
}
impl Store {
    pub(crate) fn query_encoding(&mut self, text: &str) -> Result<Arc<Vec<f32>>> {
        if let Some(vector) = self.retrieval_memo.encodings.get(text) {
            return Ok(vector.clone());
        }
        let vector = Arc::new(self.encoder()?.encode(text));
        // Work budgets only: uncached requests use exactly the same encoder.
        if self.retrieval_memo.encodings.len() < 16
            && text.len() <= 64 * 1024 - self.retrieval_memo.key_bytes
        {
            self.retrieval_memo.key_bytes += text.len();
            self.retrieval_memo
                .encodings
                .insert(text.into(), vector.clone());
        }
        Ok(vector)
    }
    pub(crate) fn block_scores(
        &mut self,
        matrix: &Arc<BlockMatrix>,
        query: &[f32],
    ) -> Arc<Vec<f32>> {
        if let Some(cached) = &self.retrieval_memo.block_scores
            && Arc::ptr_eq(&cached.matrix, matrix)
            && cached.query.len() == query.len()
            && cached
                .query
                .iter()
                .zip(query)
                .all(|(bits, v)| *bits == v.to_bits())
        {
            return cached.values.clone();
        }
        let values = Arc::new(matrix.matrix.multiply(query));
        self.retrieval_memo.block_scores = Some(BlockScores {
            matrix: matrix.clone(),
            query: query.iter().map(|v| v.to_bits()).collect(),
            values: values.clone(),
        });
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retrieval::similarity::Matrix;
    fn matrix() -> Arc<BlockMatrix> {
        Arc::new(BlockMatrix {
            ids: vec![2, 7],
            records: vec![0, 1],
            eligible: vec![true, true],
            expires: vec![String::new(), String::new()],
            matrix: Matrix::owned(vec![vec![0.125; 512], vec![-0.25; 512]]),
        })
    }
    #[tokio::test]
    async fn scores_reuse_only_identical_bits_and_matrix_identity() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("memo.db")).await.unwrap();
        let m = matrix();
        let q = vec![0.25; 512];
        let first = store.block_scores(&m, &q);
        let again = store.block_scores(&m, &q);
        assert!(Arc::ptr_eq(&first, &again));
        assert_eq!(
            first.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            m.matrix
                .multiply(&q)
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        let other_matrix = store.block_scores(&matrix(), &q);
        assert!(!Arc::ptr_eq(&first, &other_matrix));
        let zero = store.block_scores(&m, &[0.; 512]);
        let negative_zero = store.block_scores(&m, &[-0.; 512]);
        assert!(!Arc::ptr_eq(&zero, &negative_zero));
        let mut nan = [0.; 512];
        nan[0] = f32::from_bits(0x7fc0_0001);
        let a = store.block_scores(&m, &nan);
        assert!(Arc::ptr_eq(&a, &store.block_scores(&m, &nan)));
        nan[0] = f32::from_bits(0x7fc0_0002);
        assert!(!Arc::ptr_eq(&a, &store.block_scores(&m, &nan)));
    }
    #[tokio::test]
    async fn writes_and_forget_drop_numerical_and_text_memos() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalidation.db");
        let record = |id: &str| {
            serde_json::from_value(serde_json::json!({"id":id,"body":"alpha beta source"})).unwrap()
        };
        let mut store = Store::build(&path, vec![record("a")]).await.unwrap();
        let old = store.block_matrix().await.unwrap();
        store.block_scores(&old, &[0.; 512]);
        store.query_encoding("alpha beta").unwrap();
        store.add(record("b")).await.unwrap();
        assert!(store.retrieval_memo.block_scores.is_none());
        assert!(store.retrieval_memo.encodings.is_empty());
        let new = store.block_matrix().await.unwrap();
        assert!(!Arc::ptr_eq(&old, &new));
        store.block_scores(&new, &[0.; 512]);
        store.query_encoding("alpha beta").unwrap();
        store.forget("b").await.unwrap();
        assert!(store.retrieval_memo.block_scores.is_none());
        assert!(store.retrieval_memo.encodings.is_empty());
    }
    #[tokio::test]
    async fn encodings_are_exact_bounded_and_dropped_on_new_request() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("encoding.db")).await.unwrap();
        let a = store.query_encoding("alpha beta").unwrap();
        assert!(Arc::ptr_eq(
            &a,
            &store.query_encoding("alpha beta").unwrap()
        ));
        let fresh = store.encoder().unwrap().encode("alpha beta");
        assert_eq!(
            a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            fresh.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        assert!(!Arc::ptr_eq(
            &a,
            &store.query_encoding(" alpha beta").unwrap()
        ));
        for i in 0..32 {
            store.query_encoding(&format!("query {i}")).unwrap();
        }
        assert_eq!(store.retrieval_memo.encodings.len(), 16);
        store.begin_query("alpha beta", &crate::storage::Context::default());
        assert!(store.retrieval_memo.encodings.is_empty());
        let huge = "x".repeat(64 * 1024 + 1);
        store.query_encoding(&huge).unwrap();
        assert!(store.retrieval_memo.encodings.is_empty());
    }
}
