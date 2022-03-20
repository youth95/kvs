mod create;
mod delete;
mod fetch_token;
mod read;
mod remote_version;
mod update;

pub use create::{CreateAction, KeyMeta};
pub use delete::DeleteAction;
pub use fetch_token::{FetchTokenAction, KVSToken};
pub use read::{CatReply, ReadAction};
pub use remote_version::RemoteVersionAction;
pub use update::UpdateAction;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Actions {
    FetchToken(FetchTokenAction),
    CreateKeyValue(CreateAction),
    CatAction(ReadAction),
    DeleteAction(DeleteAction),
    UpdateAction(UpdateAction),
    RemoteVersionAction(RemoteVersionAction),
}
