//!
//! Bonsai - Behavior Tree
//!
//! You can serialize the
//! behavior tree using [Serde](https://crates.io/crates/serde) and
//! e.g. [Ron](https://crates.io/crates/ron).

//! A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.

//! ### How to use a Behavior tree?

//! An AI behavior tree is a very generic way of organizing interactive logic.
//! It has built-in semantics for processes that signals `Running`, `Success` or
//! `Failure`.

//! For example, if you have a state `A` and a state `B`:

//! - Move from state `A` to state `B` if `A` succeeds: `Sequence([A, B])`
//! - Try `A` first and then try `B` if `A` fails: `Select([A, B])`
//! - If `condition` succeedes do `A`, else do `B` : `If(condition, A, B)`
//! - If `A` succeeds, return failure (and vice-versa): `Invert(A)`
//! - Do `B` repeatedly while `A` runs: `While(A, [B])`
//! - Do `A`, `B` forever: `While(WaitForever, [A, B])`
//! - Wait for both `A` and `B` to complete: `WhenAll([A, B])`
//! - Wait for either `A` or `B` to complete: `WhenAny([A, B])`
//! - Wait for either `A` or `B` to complete: `WhenAny([A, B])`

//! See the `Behavior` enum for more information.

//! ## Example of use

//! This is a enemy NPC (non-player-character) behavior mock-up which decides if the AI should shoot while running for nearby cover, rush in to attack the player up close or stand its ground while firing at the player.

//! ```rust
//! use bonsai_bt::{Event, Success, UpdateArgs, BT};

//! // Some test actions.
//! #[derive(Clone, Debug)]
//! pub enum Actions {
//!     ///! Increment accumulator.
//!     Inc,
//!     ///! Decrement accumulator.
//!     Dec,
//! }

//! // A test state machine that can increment and decrement.
//! fn tick(mut acc: i32, dt: f64, bt: &mut BT<Actions, String, i32>) -> i32 {
//!     let e: Event = UpdateArgs { dt }.into();

//!     let (_status, _dt) = bt.state.tick(&e, &mut |args| match *args.action {
//!         Inc => {
//!             acc += 1;
//!             (Success, args.dt)
//!         }
//!         Dec => {
//!             acc -= 1;
//!             (Success, args.dt)
//!         }
//!     });

//!     //! update counter in blackboard
//!     let bb = bt.get_blackboard();

//!     bb.get_db()
//!         .entry("count".to_string())
//!         .and_modify(|count| *count = acc)
//!         .or_insert(0)
//!         .to_owned()

//!     acc
//! }

//! fn main() {
//!     use std::collections::HashMap;
//!     use bonsai_bt::{Action, Sequence, Wait};

//!     //! create the behavior
//!     let behavior = Sequence(vec![
//!         Wait(1.0),
//!         Action(Inc),
//!         Wait(1.0),
//!         Action(Inc),
//!         Wait(0.5),
//!         Action(Dec),
//!     ]);

//!     //! you have to initialize a blackboard even though you're
//!     //! not necessarily using it for storage
//!     let mut blackboard: HashMap<String, f32> = HashMap::new();

//!     //! instantiate the bt
//!     let mut bt = BT::new(behavior, blackboard);

//!     let a: i32 = 0;
//!     let a = tick(a, 0.5, &mut bt); //! have bt advance 0.5 seconds into the future
//!     assert_eq!(a, 0);
//!     let a = tick(a, 0.5, &mut bt); //! have bt advance another 0.5 seconds into the future
//!     assert_eq!(a, 1);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 1);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 2);
//!     let a = tick(a, 0.5, &mut bt);
//!     assert_eq!(a, 1);

//!     let bb = bt.get_blackboard();
//!     let count = bb.get_db().get("count").unwrap();
//!     assert_eq!(*count, 1);
//! }
//! ```

pub use behavior::Behavior::{
    self, Action, After, AlwaysSucceed, If, Invert, Select, Sequence, Wait, WaitForever, WhenAll, WhenAny, While,
};

pub use bt::BT;
pub use event::{Event, Timer, UpdateArgs, UpdateEvent};
pub use state::{ActionArgs, State, RUNNING};
pub use status::Status::{self, Failure, Running, Success};

mod behavior;
mod bt;
mod event;
mod sequence;
mod state;
mod status;
mod when_all;
