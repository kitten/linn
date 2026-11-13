//! Request-local output controls; never change retrieval algorithms.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    pub startup: bool,
    pub pos: bool,
    pub trace: bool,
    pub ranks: bool,
}
impl Diagnostics {
    pub fn from_environment() -> Self {
        Self {
            startup: std::env::var_os("LINN_STARTUP_TIMINGS").is_some(),
            pos: std::env::var_os("LINN_POS_TIMINGS").is_some(),
            trace: std::env::var_os("LINN_POS_TRACE").is_some(),
            ranks: std::env::var_os("LINN_POS_RANK_TRACE").is_some(),
        }
    }
}
tokio::task_local! {static CURRENT:(Diagnostics,RefCell<Vec<String>>);}
pub fn enabled(name: &str) -> bool {
    CURRENT
        .try_with(|(flags, _)| match name {
            "LINN_STARTUP_TIMINGS" => flags.startup,
            "LINN_POS_TIMINGS" => flags.pos,
            "LINN_POS_TRACE" => flags.trace,
            "LINN_POS_RANK_TRACE" => flags.ranks,
            _ => false,
        })
        .unwrap_or_else(|_| std::env::var_os(name).is_some())
}
pub fn emit(message: String) {
    if CURRENT
        .try_with(|(_, output)| output.borrow_mut().push(message.clone()))
        .is_err()
    {
        eprintln!("{message}");
    }
}
pub async fn capture<T>(
    flags: Diagnostics,
    future: impl std::future::Future<Output = T>,
) -> (T, Vec<String>) {
    CURRENT
        .scope((flags, RefCell::new(Vec::new())), async {
            let value = future.await;
            let lines = CURRENT.with(|(_, lines)| lines.take());
            (value, lines)
        })
        .await
}
