//! Original structural source-unit inventory, no source_path access.
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let r: linn::record::Record = serde_json::from_str(&line?)?;
        for section in linn::sections::parse_sections(&r.body) {
            let blocks = linn::sections::block_spans(&r.body, section.span.clone(), 1400);
            for unit in linn::sections::block_spans(&r.body, section.span, 0) {
                let text = &r.body[unit.clone()];
                if text.trim_start().starts_with("```") || text.trim_start().starts_with("~~~") {
                    continue;
                }
                let block = blocks
                    .iter()
                    .position(|b| b.start <= unit.start && unit.end <= b.end);
                println!(
                    "{}",
                    serde_json::json!({"record":r.id,"body_hash":linn::record::hash(r.body.as_bytes()),"section":section.ordinal,"block":block,"base":unit.start,"end":unit.end,"text":text})
                );
            }
        }
    }
    Ok(())
}
