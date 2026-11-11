//! Original-byte Markdown segmentation.
//! @ref LLP 0001#byte-spans

use regex::Regex;
use std::{ops::Range, sync::LazyLock};

static HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$").unwrap());
static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ ]{0,3}(`{3,}|~{3,})").unwrap());
static LIST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(?:[-+*]|\d+[.)])\s+").unwrap());
static CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").unwrap());

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub ordinal: usize,
    pub level: usize,
    pub heading: String,
    pub path: Vec<String>,
    pub span: Range<usize>,
}

fn fence_mark(line: &str) -> Option<(u8, usize)> {
    let captures = FENCE.captures(line)?;
    let mark = captures.get(1)?.as_str();
    Some((mark.as_bytes()[0], mark.len()))
}

/// Heading lines are part of the new section, including their original newline.
pub fn parse_sections(body: &str) -> Vec<Section> {
    let mut sections = Vec::<Section>::new();
    let mut ancestors = Vec::<(usize, String)>::new();
    let mut fence = None;
    let mut position = 0;
    for line in body.split_inclusive('\n') {
        let mark = fence_mark(line);
        if let Some((character, length)) = fence {
            if mark.is_some_and(|(c, n)| c == character && n >= length) {
                fence = None;
            }
        } else if mark.is_some() {
            fence = mark;
        } else if let Some(captures) = HEADING.captures(line.strip_suffix('\n').unwrap_or(line)) {
            let level = captures[1].len();
            let heading = captures[2].trim().to_owned();
            if sections.is_empty() && position > 0 {
                sections.push(Section {
                    ordinal: 0,
                    level: 0,
                    heading: String::new(),
                    path: vec![],
                    span: 0..position,
                });
            }
            if let Some(previous) = sections.last_mut() {
                previous.span.end = position;
            }
            while ancestors.last().is_some_and(|(l, _)| *l >= level) {
                ancestors.pop();
            }
            ancestors.push((level, heading.clone()));
            sections.push(Section {
                ordinal: sections.len(),
                level,
                heading,
                path: ancestors.iter().map(|(_, title)| title.clone()).collect(),
                span: position..body.len(),
            });
        }
        position += line.len();
    }
    if sections.is_empty() {
        sections.push(Section {
            ordinal: 0,
            level: 0,
            heading: String::new(),
            path: vec![],
            span: 0..body.len(),
        });
    }
    sections
}

/// Greedily packs structural units; a single oversized unit remains whole.
pub fn block_spans(body: &str, span: Range<usize>, limit: usize) -> Vec<Range<usize>> {
    let mut cuts = vec![span.start];
    let mut position = span.start;
    let mut fence = None;
    for line in byte_lines(&body[span.clone()]) {
        let mark = fence_mark(line);
        if let Some((character, length)) = fence {
            if mark.is_some_and(|(c, n)| c == character && n >= length) {
                fence = None;
                cuts.push(position + line.len());
            }
        } else if let Some(mark) = mark {
            cuts.push(position);
            fence = Some(mark);
        } else if line.trim().is_empty() {
            cuts.push(position + line.len());
        } else if LIST.is_match(line) {
            cuts.push(position);
        }
        position += line.len();
    }
    cuts.push(span.end);
    cuts.sort_unstable();
    cuts.dedup();
    let mut output = Vec::new();
    let (mut start, mut previous) = (span.start, span.start);
    for end in cuts.into_iter().skip(1) {
        if end - start > limit && previous > start {
            output.push(start..previous);
            start = previous;
        }
        previous = end;
    }
    if start < span.end {
        output.push(start..span.end);
    }
    output
}

pub fn pieces(body: &str) -> String {
    CAMEL.replace_all(body, "$1 $2").replace(['_', '-'], " ")
}

/// This cutter is intentionally byte based and independent of index blocks.
pub fn hint_spans(body: &[u8], span: Range<usize>) -> Vec<Range<usize>> {
    let mut result = Vec::new();
    let mut start = span.start;
    while start < span.end {
        let mut end = (start + 1400).min(span.end);
        if end < span.end
            && let Some(newline) = body[start + 1..end].iter().rposition(|b| *b == b'\n')
        {
            end = start + 1 + newline + 1;
        }
        result.push(start..end);
        start = end;
    }
    result
}

// Python bytes.splitlines recognizes CR, LF and CRLF (unlike str.splitlines).
fn byte_lines(text: &str) -> Vec<&str> {
    let data = text.as_bytes();
    let (mut start, mut i) = (0, 0);
    let mut lines = Vec::new();
    while i < data.len() {
        if data[i] == b'\r' || data[i] == b'\n' {
            if data[i] == b'\r' && data.get(i + 1) == Some(&b'\n') {
                i += 1;
            }
            i += 1;
            lines.push(&text[start..i]);
            start = i;
        } else {
            i += 1;
        }
    }
    if start < data.len() {
        lines.push(&text[start..]);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestry_offsets_and_original_bytes() {
        let body = "préface\n# Root ##\ntext\n### Child\nx\n## Peer\ny\n";
        let sections = parse_sections(body);
        assert_eq!(sections.len(), 4);
        assert_eq!(&body[sections[0].span.clone()], "préface\n");
        assert_eq!(sections[2].path, ["Root", "Child"]);
        assert_eq!(sections[3].path, ["Root", "Peer"]);
        assert_eq!(
            sections
                .iter()
                .map(|s| &body[s.span.clone()])
                .collect::<String>(),
            body
        );
        assert_eq!(parse_sections("# Root\n").len(), 1);
        assert_eq!(parse_sections("")[0].span, 0..0);
    }

    #[test]
    fn fences_require_matching_mark_and_length() {
        let body = "# A\n````rust\n# hidden\n```\n# still hidden\n~~~~\n````\n# B\n    ```\n# C\n";
        let sections = parse_sections(body);
        assert_eq!(
            sections
                .iter()
                .map(|s| s.heading.as_str())
                .collect::<Vec<_>>(),
            ["A", "B", "C"]
        );
    }

    #[test]
    fn packing_keeps_fences_and_oversized_units() {
        let body = format!(
            "intro\n\n```\n{}\n```\n\n- next\n- last\n",
            "x".repeat(1600)
        );
        let spans = block_spans(&body, 0..body.len(), 1400);
        assert_eq!(
            spans.iter().map(|s| &body[s.clone()]).collect::<String>(),
            body
        );
        assert!(
            spans
                .iter()
                .any(|s| body[s.clone()].starts_with("```") && body[s.clone()].ends_with("```\n"))
        );
        assert!(spans.iter().any(|s| s.len() > 1400));
    }

    #[test]
    fn hint_cuts_can_split_utf8_but_index_cuts_cannot() {
        let body = "é".repeat(1000);
        assert_eq!(hint_spans(body.as_bytes(), 0..body.len()).len(), 2);
        let blocks = block_spans(&body, 0..body.len(), 1400);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], 0..body.len());
        assert_eq!(
            pieces("pinnedVersion foo-bar MEM_KEY"),
            "pinned Version foo bar MEM KEY"
        );
    }
}
