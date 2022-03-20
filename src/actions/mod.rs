mod read;
mod create;
mod fetch_token;

pub use read::{CatReply, ReadAction};
pub use create::{CreateAction, KeyMeta};
pub use fetch_token::{FetchTokenAction, KVSToken};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Actions {
    FetchToken(FetchTokenAction),
    CreateKeyValue(CreateAction),
    CatAction(ReadAction),
}
