#[macro_use]
use serde;
// use serde_derive;

pub use behavior::Behavior::{
    self, Action, After, AlwaysSucceed, Fail, If, Select, Sequence, Wait, WaitForever, WhenAll, WhenAny, While,
};

pub use state::{ActionArgs, State, RUNNING};
pub use status::Status::{self, Failure, Running, Success};

mod behavior;
mod state;
mod status;
