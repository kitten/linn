//! Exact float32 kernels with reusable row-major storage.
//! @ref LLP 0002#static-vectors
#[cfg(target_os = "macos")]
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {}

pub(crate) struct Matrix {
    values: Vec<f32>,
    columns: usize,
    rows: usize,
}
impl Matrix {
    pub(crate) fn from_values(values: Vec<f32>, rows: usize) -> Option<Self> {
        (values.len() == rows.checked_mul(512)?).then_some(Self {
            values,
            columns: if rows == 0 { 0 } else { 512 },
            rows,
        })
    }
    pub(crate) fn row(&self, index: usize) -> &[f32] {
        &self.values[index * self.columns..(index + 1) * self.columns]
    }
    pub(crate) fn row_count(&self) -> usize {
        self.rows
    }
    pub(crate) fn chunks(&self) -> impl Iterator<Item = &[f32]> {
        self.values.chunks(16384 * 512)
    }
    #[cfg(test)]
    pub(crate) fn new<'a>(rows: impl Iterator<Item = &'a [f32]>) -> Self {
        let rows: Vec<_> = rows.collect();
        let columns = rows.first().map_or(0, |row| row.len());
        assert!(rows.iter().all(|row| row.len() == columns));
        Self {
            values: rows.iter().flat_map(|row| row.iter().copied()).collect(),
            columns,
            rows: rows.len(),
        }
    }
    pub(crate) fn owned(rows: Vec<Vec<f32>>) -> Self {
        let columns = rows.first().map_or(0, Vec::len);
        let count = rows.len();
        let mut values = Vec::with_capacity(count.checked_mul(columns).expect("matrix size"));
        for row in rows {
            assert_eq!(row.len(), columns);
            values.extend(row);
        }
        Self {
            values,
            columns,
            rows: count,
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.rows == 0
    }
    pub(crate) fn dictionary(&self) -> (Self, Vec<u32>) {
        let mut unique: std::collections::HashMap<Vec<u32>, u32> = std::collections::HashMap::new();
        let mut order = Vec::with_capacity(self.rows);
        for index in 0..self.rows {
            let bits: &[u32] = bytemuck::cast_slice(self.row(index));
            let ordinal = if let Some(ordinal) = unique.get(bits) {
                *ordinal
            } else {
                let ordinal = u32::try_from(unique.len()).expect("dictionary rows");
                unique.insert(bits.to_vec(), ordinal);
                ordinal
            };
            order.push(ordinal);
        }
        let mut entries: Vec<_> = unique.into_iter().collect();
        entries.sort_unstable_by_key(|(_, ordinal)| *ordinal);
        let rows = entries.len();
        let mut values = Vec::with_capacity(rows * self.columns);
        for (bits, _) in entries {
            values.extend(bits.into_iter().map(f32::from_bits));
        }
        (
            Self {
                values,
                columns: self.columns,
                rows,
            },
            order,
        )
    }
    pub(crate) fn expanded(&self, order: &[u32]) -> Option<Self> {
        if !order.is_empty() && self.columns != 512 {
            return None;
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(order.len().checked_mul(512)?)
            .ok()?;
        for ordinal in order {
            if *ordinal as usize >= self.rows {
                return None;
            }
            values.extend_from_slice(self.row(*ordinal as usize));
        }
        Self::from_values(values, order.len())
    }
    pub(crate) fn multiply(&self, q: &[f32]) -> Vec<f32> {
        if self.is_empty() {
            return vec![];
        }
        assert_eq!(self.columns, q.len());
        #[cfg(target_os = "macos")]
        {
            let mut out = vec![0.; self.rows];
            // SAFETY: contiguous row-major m*k matrix, k input and m output floats.
            unsafe {
                cblas::sgemv(
                    cblas::Layout::RowMajor,
                    cblas::Transpose::None,
                    i32::try_from(self.rows).expect("matrix rows"),
                    i32::try_from(q.len()).expect("matrix columns"),
                    1.,
                    &self.values,
                    q.len() as i32,
                    q,
                    1,
                    0.,
                    &mut out,
                    1,
                );
            }
            out
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.values
                .chunks_exact(self.columns)
                .map(|row| dot(row, q))
                .collect()
        }
    }
    pub(crate) fn pair(&self, a: &[f32], b: &[f32]) -> Vec<[f32; 2]> {
        if self.is_empty() {
            return vec![];
        }
        assert_eq!(a.len(), b.len());
        assert_eq!(self.columns, a.len());
        #[cfg(target_os = "macos")]
        {
            let query: Vec<_> = a.iter().chain(b).copied().collect();
            let mut out = vec![0.; self.rows * 2];
            // SAFETY: A is m*k, transposed B is k*2, and C has m*2 entries.
            unsafe {
                cblas::sgemm(
                    cblas::Layout::RowMajor,
                    cblas::Transpose::None,
                    cblas::Transpose::Ordinary,
                    i32::try_from(self.rows).expect("matrix rows"),
                    2,
                    i32::try_from(a.len()).expect("matrix columns"),
                    1.,
                    &self.values,
                    a.len() as i32,
                    &query,
                    a.len() as i32,
                    0.,
                    &mut out,
                    2,
                );
            }
            out.chunks_exact(2).map(|row| [row[0], row[1]]).collect()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.values
                .chunks_exact(self.columns)
                .map(|row| [dot(row, a), dot(row, b)])
                .collect()
        }
    }
}
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    #[cfg(target_os = "macos")]
    {
        // SAFETY: both vectors have the validated length and unit stride.
        unsafe { cblas::sdot(i32::try_from(a.len()).expect("vector length"), a, 1, b, 1) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        a.iter().zip(b).map(|(a, b)| a * b).sum()
    }
}
#[cfg(test)]
pub fn matrix<'a>(rows: impl Iterator<Item = &'a [f32]>, q: &[f32]) -> Vec<f32> {
    Matrix::new(rows).multiply(q)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dictionary_preserves_every_float_bit_and_occurrence() {
        let a = vec![0.; 512];
        let mut b = a.clone();
        b[0] = -0.;
        b[1] = f32::from_bits(0x7fc01234);
        let original = Matrix::owned(vec![a.clone(), b.clone(), a, b]);
        let (dictionary, order) = original.dictionary();
        assert_eq!(dictionary.row_count(), 2);
        assert_eq!(order, [0, 1, 0, 1]);
        let expanded = dictionary.expanded(&order).unwrap();
        assert_eq!(
            bytemuck::cast_slice::<f32, u32>(&original.values),
            bytemuck::cast_slice::<f32, u32>(&expanded.values)
        );
        assert!(dictionary.expanded(&[2]).is_none());
        let empty = Matrix::owned(vec![]);
        assert!(empty.expanded(&[]).unwrap().is_empty());
        assert!(empty.expanded(&[0]).is_none());
    }
    #[test]
    fn packed_reuse_keeps_kernel_shapes_and_float32_bits() {
        for n in [0, 1, 7, 31, 1024, 1025] {
            let rows: Vec<Vec<f32>> = (0..n)
                .map(|r| {
                    (0..512)
                        .map(|c| {
                            if r % 11 == 0 {
                                0.
                            } else {
                                ((r * 37 + c * 13) as f32).sin()
                            }
                        })
                        .collect()
                })
                .collect();
            let a: Vec<_> = (0..512).map(|i| (i as f32).cos()).collect();
            let b: Vec<_> = a.iter().map(|v| -*v).collect();
            let packed = Matrix::new(rows.iter().map(Vec::as_slice));
            let (dictionary, order) = packed.dictionary();
            let expanded = dictionary.expanded(&order).unwrap();
            let owned = Matrix::owned(rows.clone());
            for query in [&a, &b, &a] {
                let fresh = matrix(rows.iter().map(Vec::as_slice), query);
                assert_eq!(
                    owned
                        .multiply(query)
                        .iter()
                        .map(|v| v.to_bits())
                        .collect::<Vec<_>>(),
                    fresh.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
                );
                assert_eq!(
                    packed
                        .multiply(query)
                        .iter()
                        .map(|v| v.to_bits())
                        .collect::<Vec<_>>(),
                    fresh.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
                );
            }
            let pair = packed.pair(&a, &b);
            assert_eq!(
                pair.iter()
                    .flatten()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                expanded
                    .pair(&a, &b)
                    .iter()
                    .flatten()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
            let fresh = Matrix::new(rows.iter().map(Vec::as_slice)).pair(&a, &b);
            assert_eq!(
                pair.iter()
                    .flatten()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                fresh
                    .iter()
                    .flatten()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
        }
    }
}
