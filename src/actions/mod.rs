mod cat;
mod create_key_value;
mod fetch_token;

pub use cat::{CatAction, CatReply};
pub use create_key_value::{CreateKeyValueAction, KeyMeta};
pub use fetch_token::{FetchTokenAction, KVSToken};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Actions {
    FetchToken(FetchTokenAction),
    CreateKeyValue(CreateKeyValueAction),
    CatAction(CatAction),
}
