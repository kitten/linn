//! Introselect posting order for deterministic unsorted top-k probes.
//! @ref LLP 0002#ordered-fallbacks
//!
//! Median-of-three selection, with a median-of-five depth fallback. Returning a
//! sorted top-k changes which equal-scoring sentence pair is encountered first.
pub fn top(scores: &[f32], take: usize) -> Vec<usize> {
    if scores.is_empty() {
        return vec![];
    }
    let values: Vec<_> = scores.iter().map(|v| -v).collect();
    let mut ids: Vec<_> = (0..scores.len()).collect();
    select(&values, &mut ids, take.min(scores.len() - 1));
    ids.truncate(take);
    ids
}
fn less(v: &[f32], ids: &[usize], a: usize, b: usize) -> bool {
    v[ids[a]] < v[ids[b]]
}
fn select(v: &[f32], ids: &mut [usize], k: usize) {
    let n = ids.len();
    let (mut low, mut high) = (0, n - 1);
    if k < 3 {
        for i in 0..=k {
            let mut best = i;
            for j in i + 1..n {
                if less(v, ids, j, best) {
                    best = j;
                }
            }
            ids.swap(i, best);
        }
        return;
    }
    if k == n - 1 {
        let mut best = 0;
        for j in 1..n {
            if !less(v, ids, j, best) {
                best = j;
            }
        }
        ids.swap(k, best);
        return;
    }
    let mut depth = (usize::BITS - n.leading_zeros() - 1) * 2;
    while low + 1 < high {
        let (mut left, mut right) = (low + 1, high);
        if depth > 0 || right - left < 5 {
            let mid = low + (high - low) / 2;
            if less(v, ids, high, mid) {
                ids.swap(high, mid);
            }
            if less(v, ids, high, low) {
                ids.swap(high, low);
            }
            if less(v, ids, low, mid) {
                ids.swap(low, mid);
            }
            ids.swap(mid, low + 1);
        } else {
            let mid = left + median_groups(v, &mut ids[left..right]);
            ids.swap(mid, low);
            left -= 1;
            right += 1;
        }
        depth = depth.saturating_sub(1);
        let pivot = v[ids[low]];
        loop {
            left += 1;
            while v[ids[left]] < pivot {
                left += 1;
            }
            right -= 1;
            while pivot < v[ids[right]] {
                right -= 1;
            }
            if right < left {
                break;
            }
            ids.swap(left, right);
        }
        ids.swap(low, right);
        if right >= k {
            high = right - 1;
        }
        if right <= k {
            low = left;
        }
    }
    if high == low + 1 && less(v, ids, high, low) {
        ids.swap(high, low);
    }
}
fn median_groups(v: &[f32], ids: &mut [usize]) -> usize {
    let n = ids.len() / 5;
    for i in 0..n {
        let base = i * 5;
        let b = &mut ids[base..base + 5];
        if less(v, b, 1, 0) {
            b.swap(1, 0);
        }
        if less(v, b, 4, 3) {
            b.swap(4, 3);
        }
        if less(v, b, 3, 0) {
            b.swap(3, 0);
        }
        if less(v, b, 4, 1) {
            b.swap(4, 1);
        }
        if less(v, b, 2, 1) {
            b.swap(2, 1);
        }
        let median = if less(v, b, 3, 2) {
            if less(v, b, 3, 1) { 1 } else { 3 }
        } else {
            2
        };
        ids.swap(base + median, i);
    }
    if n > 2 {
        select(v, &mut ids[..n], n / 2);
    }
    n / 2
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probes_are_top_k_with_equal_scores_and_depth_fallback() {
        for n in [1, 3, 16, 21, 128, 4096] {
            let scores: Vec<_> = (0..n).map(|i| ((i * 101) % 37) as f32).collect();
            let k = n.min(15);
            let selected = top(&scores, k);
            let mut expected = scores.clone();
            expected.sort_by(|a, b| b.total_cmp(a));
            let mut got: Vec<_> = selected.iter().map(|i| scores[*i]).collect();
            got.sort_by(|a, b| b.total_cmp(a));
            assert_eq!(got, expected[..k]);
        }
    }
}
