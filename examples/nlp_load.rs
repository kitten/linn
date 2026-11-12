//! Fresh-process embedded NLP loading probe; excludes annotation and destruction.
use std::time::Instant;
fn main() {
    let started = Instant::now();
    let tokenizer = linn::nlp::Tokenizer::from_compiled(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/english.nlp"
    )))
    .unwrap();
    println!("{:.3}", started.elapsed().as_secs_f64() * 1000.);
    std::hint::black_box(tokenizer);
}
