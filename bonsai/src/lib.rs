//!
//! *Bonsai - Behavior Tree*
//!
//! You can serialize the
//! behavior tree using [Serde](https://crates.io/crates/serde),
//! [Ron](https://crates.io/crates/ron) and [graphviz](https://graphviz.org/)
//!
//! A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.
//!
//! ### How to use a Behavior tree?

//! A Behavior Tree forms a tree structure where each node represents a process.
//! When the process terminates, it signals `Success` or `Failure`. This can then
//! be used by the parent node to select the next process.
//! A signal `Running` is used to tell the process is not done yet.

//! For example, if you have a state `A` and a state `B`:

//! - Move from state `A` to state `B` if `A` succeeds: `Sequence([A, B])`
//! - Try `A` first and then try `B` if `A` fails: `Select([A, B])`
//! - If `condition` succeedes do `A`, else do `B` : `If(condition, A, B)`
//! - If `A` succeeds, return failure (and vice-versa): `Invert(A)`
//! - Do `B` repeatedly while `A` runs: `While(A, [B])`
//! - Do `A`, `B` forever: `While(WaitForever, [A, B])`
//! - Run `A` and `B` in parallell and wait for both to succeed: `WhenAll([A, B])`
//! - Run `A` and `B` in parallell and wait for any to succeed: `WhenAny([A, B])`
//! - Run `A` and `B` in parallell, but `A` has to succeed before `B`: `After([A, B])`
//!
//! See the `Behavior` enum for more information.

//! ## Example of use

//! This is a simple example with two possible Actions: Increment a number, Decrement a number. We
//! construct a BT where we increment a number twice, one second apart. Then wait 0.5 seconds before we
//! then decrement the same number again. Additionally we use a Blackboard to store/persist the immediate
//! state of the number accessed by the key `count`.
//!
//! ```rust
//! use bonsai_bt::{Event, Success, UpdateArgs, BT};
//! use std::collections::HashMap;
//! // Some test actions.
//! #[derive(Clone, Debug, Copy)]
//! pub enum Actions {
//!     ///! Increment accumulator.
//!     Inc,
//!     ///! Decrement accumulator.
//!     Dec,
//! }
//!
//! // A test state machine that can increment and decrement.
//! fn tick(mut acc: i32, dt: f64, bt: &mut BT<Actions, HashMap<String, i32>>) -> i32 {
//! let e: Event = UpdateArgs { dt }.into();
//!
//!     let (_status, _dt) = bt.tick(&e, &mut |args, blackboard| match *args.action {
//!         Actions::Inc => {
//!             acc += 1;
//!             (Success, args.dt)
//!         }
//!         Actions::Dec => {
//!             acc -= 1;
//!             (Success, args.dt)
//!         }
//!     }).unwrap();
//!
//!     // update counter in blackboard
//!     let bb = bt.blackboard_mut();
//!
//!     bb.entry("count".to_string())
//!         .and_modify(|count| *count = acc)
//!         .or_insert(0)
//!         .to_owned();
//!
//!     acc
//! }
//!
//! fn main() {
//!     use crate::Actions::{Inc, Dec};
//!     use std::collections::HashMap;
//!     use bonsai_bt::{Action, Sequence, Wait};
//!
//!     // create the behavior
//!     let behavior = Sequence(vec![
//!         Wait(1.0),
//!         Action(Inc),
//!         Wait(1.0),
//!         Action(Inc),
//!         Wait(0.5),
//!         Action(Dec),
//!     ]);
//!
//!     // you have to initialize a blackboard even though you're
//!     // not necessarily using it for storage
//!     let mut blackboard: HashMap<String, i32> = HashMap::new();
//!
//!     // instantiate the bt
//!     let mut bt = BT::new(behavior, blackboard);
//!
//!     let a: i32 = 0;
//!     let a = tick(a, 0.5, &mut bt); // have bt advance 0.5 seconds into the future
//!     assert_eq!(a, 0);
//!     let a = tick(a, 0.5, &mut bt); // have bt advance another 0.5 seconds into the future
//!     assert_eq!(a, 1);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 1);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 2);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 1);
//!
//!     let bb = bt.blackboard_mut();
//!     let count = bb.get("count").unwrap();
//!     assert_eq!(*count, 1);
//!
//!     // if the behavior tree concludes (reaches a steady state)
//!     // you can reset the tree back to it's initial state at t=0.0
//!     bt.reset_bt();
//! }
//! ```

pub use behavior::Behavior::{
    self, Action, After, AlwaysSucceed, If, Invert, Select, Sequence, Wait, WaitForever, WhenAll, WhenAny, While,
    WhileAll,
};

pub use bt::BT;
pub use event::{Event, Timer, UpdateArgs, UpdateEvent};
pub use state::{ActionArgs, RUNNING};
pub use status::Status::{self, Failure, Running, Success};

mod behavior;
mod bt;
mod event;
mod sequence;
mod state;
mod status;
mod when_all;

#[cfg(feature = "visualize")]
mod visualizer;
