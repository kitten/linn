//! Introselect posting order for deterministic unsorted top-k probes.
//! @ref LLP 0002#ordered-fallbacks

pub fn top(scores: &[f32], take: usize) -> Vec<usize> {
    let n = scores.len();
    let take = take.min(n);
    if take == 0 {
        return vec![];
    }
    let target = take.min(n - 1);
    let values: Vec<f32> = scores.iter().map(|x| -x).collect();
    let mut ids: Vec<usize> = (0..n).collect();
    partition(&values, &mut ids, target);

    ids.truncate(take);
    ids
}

fn partition(values: &[f32], ids: &mut [usize], target: usize) {
    let n = ids.len();
    if n < 2 {
        return;
    }
    if target < 3 {
        for i in 0..=target {
            let mut minimum = i;
            for j in i + 1..n {
                if values[ids[j]] < values[ids[minimum]] {
                    minimum = j;
                }
            }
            ids.swap(i, minimum);
        }
        return;
    }
    if target == n - 1 {
        let mut last_maximum = 0;
        for j in 1..n {
            let is_smaller = values[ids[j]] < values[ids[last_maximum]];
            // Ties and unordered comparisons both replace the current maximum.
            if !is_smaller {
                last_maximum = j;
            }
        }
        ids.swap(last_maximum, n - 1);
        return;
    }
    let mut depth = 2 * n.ilog2();
    let mut lo = 0;
    let mut hi = n - 1;
    while lo + 1 < hi {
        let mut forward;
        let mut backward;
        if depth > 0 || hi - lo < 6 {
            let mid = lo + (hi - lo) / 2;
            if values[ids[hi]] < values[ids[mid]] {
                ids.swap(hi, mid);
            }
            if values[ids[hi]] < values[ids[lo]] {
                ids.swap(hi, lo);
            }
            if values[ids[lo]] < values[ids[mid]] {
                ids.swap(lo, mid);
            }
            ids.swap(mid, lo + 1);
            forward = lo + 1;
            backward = hi;
        } else {
            let position = compact_groups(values, &mut ids[lo + 1..hi]);
            ids.swap(lo, lo + 1 + position);
            forward = lo;
            backward = hi + 1;
        }
        depth = depth.saturating_sub(1);
        let pivot = values[ids[lo]];
        loop {
            forward += 1;
            while values[ids[forward]] < pivot {
                forward += 1;
            }
            backward -= 1;
            while pivot < values[ids[backward]] {
                backward -= 1;
            }
            if backward < forward {
                break;
            }
            ids.swap(forward, backward);
        }
        ids.swap(lo, backward);
        if backward >= target {
            hi = backward - 1;
        }
        // Equality deliberately updates both bounds.
        if backward <= target {
            lo = forward;
        }
    }
    if lo + 1 == hi && values[ids[hi]] < values[ids[lo]] {
        ids.swap(lo, hi);
    }
}

fn compact_groups(values: &[f32], ids: &mut [usize]) -> usize {
    let count = ids.len() / 5;
    for group in 0..count {
        let base = 5 * group;
        for (left, right) in [(0, 1), (3, 4), (0, 3), (1, 4), (1, 2)] {
            if values[ids[base + right]] < values[ids[base + left]] {
                ids.swap(base + left, base + right);
            }
        }
        // The final decision selects a slot without further mutation.
        let median = if values[ids[base + 3]] < values[ids[base + 2]] {
            if values[ids[base + 3]] < values[ids[base + 1]] {
                1
            } else {
                3
            }
        } else {
            2
        };
        ids.swap(group, base + median);
    }
    if count > 2 {
        partition(values, &mut ids[..count], count / 2);
    }
    count / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_compaction_preserves_residue_and_ties() {
        for (values, expected) in [
            ([0., 1., 2., 3., 4.], [2, 1, 0, 3, 4]),
            ([0., 3., 4., 1., 2.], [4, 0, 2, 3, 1]),
            ([0.; 5], [2, 1, 0, 3, 4]),
        ] {
            let mut ids = [0, 1, 2, 3, 4];
            assert_eq!(compact_groups(&values, &mut ids), 0);
            assert_eq!(ids, expected);
        }
    }

    #[test]
    fn two_medians_stay_in_extraction_order_and_tail_is_ignored() {
        let values: Vec<_> = (0..12).rev().map(|i| i as f32).collect();
        let mut ids: Vec<_> = (0..12).collect();
        assert_eq!(compact_groups(&values, &mut ids), 1);
        assert_eq!(ids, [2, 7, 4, 1, 0, 9, 8, 3, 6, 5, 10, 11]);
        assert!(values[ids[0]] > values[ids[1]]);
    }

    #[test]
    fn recursive_median_prefix_preserves_displaced_ids() {
        let values: Vec<_> = (0..15).rev().map(|i| i as f32).collect();
        let mut ids: Vec<_> = (0..15).collect();
        assert_eq!(compact_groups(&values, &mut ids), 1);
        assert_eq!(ids, [12, 7, 2, 1, 0, 9, 8, 3, 6, 5, 14, 13, 4, 11, 10]);
    }

    #[test]
    fn deep_partitions_keep_exact_probe_order() {
        let scores: Vec<_> = (0..76).map(|i| i as f32).collect();
        assert_eq!(top(&scores, 3), [74, 75, 73]);
        let scores: Vec<_> = (0..80).map(|i| i as f32).collect();
        assert_eq!(
            top(&scores, 15),
            [72, 79, 78, 77, 76, 75, 74, 73, 71, 69, 70, 68, 67, 66, 65]
        );
        let scores: Vec<_> = (0..127).map(|i| i as f32).collect();
        assert_eq!(
            top(&scores, 15),
            [
                125, 121, 122, 126, 123, 124, 120, 118, 119, 117, 116, 115, 114, 112, 113
            ]
        );
    }

    #[test]
    fn empty_zero_and_oversized_probes_keep_their_contract() {
        assert!(top(&[], 15).is_empty());
        assert!(top(&[1., 2.], 0).is_empty());
        assert_eq!(top(&[0., 1.], 3), [1, 0]);
    }

    #[test]
    fn final_position_shortcut_preserves_unordered_comparisons() {
        for bits in [0x7fc0_0001, 0x7f80_0001, 0xffc0_0001, 0xff80_0001] {
            let scores = [0., f32::from_bits(bits), 2., 1.];
            assert_eq!(top(&scores, 4), [0, 1, 2, 3]);
        }
        assert_eq!(top(&[0., -0., 0., -0.], 4), [0, 1, 2, 3]);
    }

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

// @ref LLP 0002#joint-record-competition
// Heapify once, then inspect only the exact score/id-ordered prefix needed by
// lifecycle, query filters and distinct-record selection.
pub(super) struct SimilarityQueue(std::collections::BinaryHeap<SimilarityRow>);
#[derive(Clone, Copy)]
struct SimilarityRow(usize, f32);
impl PartialEq for SimilarityRow {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for SimilarityRow {}
impl PartialOrd for SimilarityRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SimilarityRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1
            .total_cmp(&other.1)
            .then_with(|| other.0.cmp(&self.0))
    }
}
impl SimilarityQueue {
    pub(super) fn new(rows: impl Iterator<Item = (usize, f32)>) -> Self {
        Self(rows.map(|(id, score)| SimilarityRow(id, score)).collect())
    }
    pub(super) fn next_batch(&mut self) -> Vec<usize> {
        (0..64)
            .map_while(|_| self.0.pop().map(|row| row.0))
            .collect()
    }
}

#[cfg(test)]
mod similarity_queue_tests {
    use super::*;
    #[test]
    fn batches_match_full_sort_including_float_ties_and_filtered_prefixes() {
        let values = [
            f32::NEG_INFINITY,
            -1.,
            -0.,
            0.,
            0.5,
            0.5,
            f32::INFINITY,
            f32::NAN,
        ];
        for n in [0, 1, 63, 64, 65, 1024] {
            let rows: Vec<_> = (0..n)
                .map(|i| (i * 7 + 3, values[(i * 13) % values.len()]))
                .filter(|(id, _)| id % 3 != 0)
                .collect();
            let mut expected = rows.clone();
            expected.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let mut queue = SimilarityQueue::new(rows.into_iter());
            for batch in expected.chunks(64) {
                assert_eq!(
                    queue.next_batch(),
                    batch.iter().map(|x| x.0).collect::<Vec<_>>()
                );
            }
            assert!(queue.next_batch().is_empty());
        }
    }
}
