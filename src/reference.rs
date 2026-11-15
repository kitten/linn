//! @ref LLP 0007#application-boundary
//! Stable original-byte fetch identifiers, independent of presentation.
pub(crate) fn fetch(hash: &str, span: [usize; 2]) -> String {
    format!("{hash}:v1:{}:{}", span[0], span[1])
}
pub(crate) fn short_fetch(hash: &str, span: [usize; 2]) -> String {
    fetch(&hash[..12], span)
}
