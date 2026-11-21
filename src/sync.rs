//! Bounded original-item wire types; peer transport lives in mesh.
pub mod protocol {
    //! @ref LLP 0008#replica-wire-contract
    use crate::storage::sync::Item;
    use serde::{Deserialize, Serialize};
    pub const MAX_FRAME: usize = 16 * 1024 * 1024;
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
    pub enum Request {
        Inventory {
            tombstones: bool,
            after: Option<String>,
        },
        Get {
            id: String,
        },
        Finish,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "result", rename_all = "snake_case")]
    pub enum Response {
        Item { item: Option<Item> },
        Done,
    }
}
