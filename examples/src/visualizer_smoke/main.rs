//! End-to-end demo for the WebSocket visualizer.
//!
//! Drives a real `BT` over a deliberately rich 27-node tree, attaches the
//! visualizer via `BT::with_telemetry(8910)`, and re-runs the tree every wall
//! tick (~400 ms). Exercises every `node_type` CSS hook, label formatter, and
//! status color, plus `tick` auto-routing to `tick_recording` (no explicit
//! telemetry call needed).
//!
//! # How to run
//!
//! ```bash
//! cargo run --bin visualizer_smoke
//! ```
//!
//! Then open <http://127.0.0.1:8910/> in a browser.
//! 1. Tree renders within ~1 s; status bar reads `connected` and `27 nodes`.
//! 2. All distinct node-type labels visible (note: `Select` shows as
//!    `Selector`, `Invert` as `Inverter` — see `classify` in `telemetry.rs`).
//! 3. `Wait` leaves display dynamic labels: `Wait(2.00s)` and `Wait(0.30s)`.
//! 4. Every ~400 ms the leaf colors shift across **all** subtrees. Every leaf
//!    cycles through Success (green), Running (yellow), and Failure (red) on
//!    a 5-step rotation with a per-action phase offset, so at any tick a mix
//!    of statuses is visible. When a leaf returns Running, the path from root
//!    to that leaf turns yellow.
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
//! Running, no visual signal) and `WhileAll` (renders identically to `While`).

use bonsai_bt::{
    Action, After, AlwaysSucceed, Behavior, Event, If, Invert, Race, Select, Sequence, Status, UpdateArgs, Wait,
    WhenAll, WhenAny, While, BT,
};
use std::time::Duration;

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
        While(Box::new(Action("has_ammo")), vec![Action("fire"), Wait(0.3)]),
        After(vec![Action("cooldown"), Action("ready_signal")]),
        WhenAny(vec![Action("victory_check"), Action("retreat_signal")]),
    ])
}

fn main() {
    let mut bt = BT::<&'static str, ()>::new(build_tree(), ())
        .with_telemetry(8910)
        .expect("bind 127.0.0.1:8910");

    // dt = 1.0 s/tick — `Wait(0.3)` fires immediately when reached;
    // `Wait(2.0)` fires after two ticks.
    let event: Event = UpdateArgs { dt: 1.0 }.into();
    let mut tick_n: u64 = 0;

    // Five-step status cycle visible across all three colors. Each action
    // gets a unique phase offset so the same wall tick produces a varied
    // mix of statuses across the tree (and yellow-Running shows up).
    const CYCLE: &[Status] = &[
        Status::Success,
        Status::Running,
        Status::Failure,
        Status::Success,
        Status::Running,
    ];

    loop {
        tick_n += 1;
        let outcome = bt.tick(&event, &mut |args, _bb| {
            let phase: u64 = match *args.action {
                "low_hp" => 0,
                "flee" => 1,
                "regroup" => 2,
                "acquire_target" => 3,
                "aim" => 4,
                "track" => 0,
                "dodge" => 1,
                "enemy_visible" => 2,
                "has_ammo" => 3,
                "fire" => 4,
                "cooldown" => 0,
                "ready_signal" => 1,
                "victory_check" => 2,
                "retreat_signal" => 3,
                _ => 0,
            };
            let idx = ((tick_n + phase) % CYCLE.len() as u64) as usize;
            let mut status = CYCLE[idx];
            // The root is a Sequence: any child returning Failure short-
            // circuits *before* downstream siblings (After at id 21, WhenAny
            // at id 24) ever get visited. Four leaves are chain-critical —
            // their Failure propagates straight up to the root:
            //   - regroup       → If's on_failure branch → If Failure
            //   - has_ammo      → While Failure
            //   - cooldown,
            //     ready_signal  → After Failure
            // Substitute Running for Failure on these so the chain reaches
            // the bottom branches. These four nodes still show Success
            // (green) and Running (yellow); the other thirteen leaves keep
            // cycling through all three statuses including red.
            if matches!(
                *args.action,
                "regroup" | "has_ammo" | "cooldown" | "ready_signal"
            ) && status == Status::Failure
            {
                status = Status::Running;
            }
            (status, 0.0)
        });
        // The tree completes in one tick (no WaitForever in the unconditional
        // path). reset_bt rewinds the cursor so the next tick runs it again;
        // tick_count and telemetry_sender survive the reset, so the browser
        // sees a continuous stream of TickTraces with monotonic tick_id.
        if matches!(outcome, Some((Status::Success | Status::Failure, _))) {
            bt.reset_bt();
        }
        std::thread::sleep(Duration::from_millis(400));
    }
}
