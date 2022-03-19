mod cat;
mod create_key_value;
mod fetch_token;

pub use cat::Cat;
pub use create_key_value::{CreateKeyValue, KeyMeta};
pub use fetch_token::{FetchToken, KVSToken};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Actions {
    FetchToken(FetchToken),
    CreateKeyValue(CreateKeyValue),
    Cat(Cat),
}
