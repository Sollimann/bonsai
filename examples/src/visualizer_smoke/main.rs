//! Manual smoke test for the WebSocket visualizer
//!
//! Spins up `spawn_server` on 127.0.0.1:8910 with a deliberately rich tree, then
//! pushes synthetic `TickTrace`s every 400 ms. We do **not** drive a real `BT` —
//! the goal is to exercise the *frontend* (every `node_type` CSS hook, label
//! formatter, status color), so the trace generator just hand-crafts plausible
//! state maps.
//!
//! # How to run
//!
//! ```bash
//! cargo run --bin visualizer_smoke
//! ```
//!
//! Then open <http://127.0.0.1:8910/> in a browser. Acceptance:
//! 1. Tree renders within ~1 s; status bar reads `connected` and `27 nodes`.
//! 2. All 12 distinct node-type labels visible (note: `Select` shows as
//!    `Selector`, `Invert` as `Inverter` — see `classify` in `telemetry.rs`).
//! 3. `Wait` leaves display dynamic labels: `Wait(2.00s)` and `Wait(0.30s)`.
//! 4. Every ~400 ms the running "context" rotates through composites/decorators
//!    while leaves cycle through Success (green) / Failure (red) / Running
//!    (amber). Root stays amber the whole time.
//! 5. Refresh the page → tree re-renders; tick_id continues monotonically.
//! 6. `Ctrl-C` the binary → status bar shows "disconnected — retrying in
//!    500ms"; restart the binary → page reconnects within ≤ 1 s.
//!
//! Hover any node to see its full label via the SVG `<title>` tooltip
//! (labels longer than 30 chars are truncated in the rendered text).
//!
//! # Tree (27 nodes, DFS preorder IDs, 12 of 14 `Behavior` variants)
//!
//! ```text
//! 0  Sequence (root)
//! ├── 1  If
//! │   ├── 2  Action("low_hp")           (cond)
//! │   ├── 3  AlwaysSucceed              (on_success)
//! │   │   └── 4  Action("flee")
//! │   └── 5  Action("regroup")          (on_failure)
//! ├── 6  Select
//! │   ├── 7  Sequence
//! │   │   ├── 8  Action("acquire_target")
//! │   │   └── 9  WhenAll
//! │   │       ├── 10 Action("aim")
//! │   │       └── 11 Action("track")
//! │   ├── 12 Race
//! │   │   ├── 13 Action("dodge")
//! │   │   └── 14 Wait(2.0)              (timeout arm)
//! │   └── 15 Invert
//! │       └── 16 Action("enemy_visible")
//! ├── 17 While
//! │   ├── 18 Action("has_ammo")         (cond)
//! │   ├── 19 Action("fire")             (body)
//! │   └── 20 Wait(0.3)                  (body)
//! ├── 21 After
//! │   ├── 22 Action("cooldown")
//! │   └── 23 Action("ready_signal")
//! └── 24 WhenAny
//!     ├── 25 Action("victory_check")
//!     └── 26 Action("retreat_signal")
//! ```
//!
//! ID assignment follows `bonsai_bt::telemetry::children_of` — `If` is
//! `[cond, ok, ko]`, `While` is `[cond, body0, body1, …]`, decorators wrap one
//! child, composites preserve order. Skipped variants: `WaitForever` (always
//! running, no visual signal) and `WhileAll` (renders identically to `While`).

use bonsai_bt::telemetry::{TickTrace, TreeDefinition};
use bonsai_bt::{
    Action, After, AlwaysSucceed, Behavior, If, Invert, Race, Select, Sequence, Status, Wait, WhenAll, WhenAny, While,
};
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

/// Leaf node IDs (DFS preorder) — every leaf in the tree above.
const LEAF_IDS: &[usize] = &[2, 4, 5, 8, 10, 11, 13, 14, 16, 18, 19, 20, 22, 23, 25, 26];
/// Inner composite/decorator IDs — rotated through as the per-tick "running context".
const INNER_IDS: &[usize] = &[1, 6, 7, 9, 12, 15, 17, 21, 24];
/// Leaf-status palette. Repeating Success twice biases the demo toward green;
/// over ~16 ticks every leaf still hits all three colors.
const STATUS_CYCLE: &[Status] = &[Status::Success, Status::Success, Status::Failure];

fn build_tree() -> Behavior<&'static str> {
    Sequence(vec![
        If(
            Box::new(Action("low_hp")),
            Box::new(AlwaysSucceed(Box::new(Action("flee")))),
            Box::new(Action("regroup")),
        ),
        Select(vec![
            Sequence(vec![
                Action("acquire_target"),
                WhenAll(vec![Action("aim"), Action("track")]),
            ]),
            Race(vec![Action("dodge"), Wait(2.0)]),
            Invert(Box::new(Action("enemy_visible"))),
        ]),
        While(
            Box::new(Action("has_ammo")),
            vec![Action("fire"), Wait(0.3)],
        ),
        After(vec![Action("cooldown"), Action("ready_signal")]),
        WhenAny(vec![Action("victory_check"), Action("retreat_signal")]),
    ])
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8910").unwrap();
    let behavior = build_tree();
    let json = serde_json::to_string(&TreeDefinition::build(&behavior)).unwrap();

    let (tx, rx) = sync_channel::<TickTrace>(1024);
    bonsai_bt::spawn_server(listener, json, rx).unwrap();

    println!("open http://127.0.0.1:8910/");

    let mut tick: u64 = 0;
    loop {
        tick += 1;
        let t = tick as usize;

        let mut states = HashMap::new();
        states.insert(0, Status::Running);
        states.insert(INNER_IDS[t % INNER_IDS.len()], Status::Running);
        for offset in 0..3 {
            let leaf = LEAF_IDS[(t + offset) % LEAF_IDS.len()];
            let status = STATUS_CYCLE[(t + offset) % STATUS_CYCLE.len()];
            states.insert(leaf, status);
        }

        tx.send(TickTrace { tick_id: tick, states }).unwrap();
        std::thread::sleep(Duration::from_millis(400));
    }
}
