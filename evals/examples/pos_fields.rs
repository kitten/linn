// Original E3 flat source parser reused for literal population inventory.
use std::{
    io::{self, BufRead},
    ops::Range,
};
#[derive(Debug)]
pub struct Field {
    pub object: Range<usize>,
    pub header: Range<usize>,
    pub value: Range<usize>,
}
fn trim_span(text: &str, start: usize) -> Range<usize> {
    let a = text.len() - text.trim_start().len();
    start + a..start + a + text.trim().len()
}
fn cells(line: &str, offset: usize) -> Option<Vec<Range<usize>>> {
    if line.len() < 2
        || !line.starts_with('|')
        || !line.ends_with('|')
        || line.contains(['\\', '`'])
    {
        return None;
    }
    let mut pos = offset + 1;
    Some(
        line[1..line.len() - 1]
            .split('|')
            .map(|s| {
                let r = trim_span(s, pos);
                pos += s.len() + 1;
                r
            })
            .collect(),
    )
}
/// Recognize only flat, unfenced, rectangular source structures. Offsets are block-relative.
pub fn fields(text: &str) -> Vec<Field> {
    let mut lines = Vec::new();
    let mut pos = 0;
    for line in text.split_inclusive('\n') {
        let s = line.trim_end_matches(['\r', '\n']);
        lines.push((s, pos));
        pos += line.len();
    }
    if text.contains("```") || text.contains("~~~") {
        return vec![];
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        if let (Some(headers), Some(separator)) = (
            cells(lines[i].0, lines[i].1),
            cells(lines[i + 1].0, lines[i + 1].1),
        ) {
            if headers.len() < 2
                || headers.len() != separator.len()
                || headers.iter().any(Range::is_empty)
                || !separator.iter().all(|r| {
                    let s = text[r.clone()].trim_matches(':');
                    s.len() >= 3 && s.chars().all(|c| c == '-')
                })
            {
                i += 1;
                continue;
            }
            let mut rows = Vec::new();
            let mut j = i + 2;
            let mut valid = true;
            while j < lines.len() && lines[j].0.starts_with('|') {
                match cells(lines[j].0, lines[j].1) {
                    Some(r) if r.len() == headers.len() => rows.push(r),
                    _ => valid = false,
                };
                j += 1;
            }
            if valid {
                for row in rows {
                    for col in 1..headers.len() {
                        out.push(Field {
                            object: row[0].clone(),
                            header: headers[col].clone(),
                            value: row[col].clone(),
                        });
                    }
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // A flat explicit list starts with an object key, followed by its actual fields.
    let mut object = None;
    for (line, pos) in lines {
        let Some(rest) = line.strip_prefix("- ") else {
            object = None;
            continue;
        };
        let Some((key, value)) = rest.split_once(": ") else {
            object = None;
            continue;
        };
        let header = trim_span(key, pos + 2);
        let value = trim_span(value, pos + 2 + key.len() + 2);
        if key == "object" {
            object = Some(value);
        } else if let Some(o) = &object {
            out.push(Field {
                object: o.clone(),
                header,
                value,
            });
        }
    }
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let r: linn::record::Record = serde_json::from_str(&line?)?;
        for section in linn::sections::parse_sections(&r.body) {
            for block in linn::sections::block_spans(&r.body, section.span, 1400) {
                let text = &r.body[block.clone()];
                for f in fields(text) {
                    println!(
                        "{}",
                        serde_json::json!({"record":r.id,"body_hash":linn::record::hash(r.body.as_bytes()),"section":section.ordinal,"block":block,"object":{"text":text[f.object.clone()],"base":block.start+f.object.start},"header":{"text":text[f.header.clone()],"base":block.start+f.header.start},"value":{"text":text[f.value.clone()],"base":block.start+f.value.start}})
                    );
                }
            }
        }
    }
    Ok(())
}
