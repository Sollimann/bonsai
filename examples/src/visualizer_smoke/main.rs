//! End-to-end demo for the WebSocket visualizer.
//!
//! Drives a real `BT` over a deliberately rich 27-node tree, attaches the
//! visualizer via `BT::with_telemetry(8910)`, and re-runs the tree every wall
//! tick (~400 ms). Exercises every `node_type` CSS hook, label formatter, and
//! status color, plus `tick` auto-routing to `tick_recording` (no explicit
//! telemetry call needed).
//!
//! Each wall tick: call `bt.tick(...)` once → the closure resolves leaves
//! based on a rotating tick counter so different `If`/`Invert` branches fire →
//! the tree returns `Success` (or `Failure`) → `reset_bt` rewinds it for the
//! next tick. This produces visible animation in the browser as leaves
//! light up green/red and the active branches shift over time.
//!
//! # How to run
//!
//! ```bash
//! cargo run --bin visualizer_smoke
//! ```
//!
//! Then open <http://127.0.0.1:8910/> in a browser. Acceptance:
//! 1. Tree renders within ~1 s; status bar reads `connected` and `27 nodes`.
//! 2. All distinct node-type labels visible (note: `Select` shows as
//!    `Selector`, `Invert` as `Inverter` — see `classify` in `telemetry.rs`).
//! 3. `Wait` leaves display dynamic labels: `Wait(2.00s)` and `Wait(0.30s)`.
//! 4. Every ~400 ms the leaf colors shift: the `If` branch alternates between
//!    `flee` and `regroup`; `Invert(enemy_visible)` flips between Success and
//!    Failure as the closure rotates the underlying status.
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
//!
//! Note: with the current closure, `has_ammo` always returns `Success`, so
//! `While` exits without entering its body — `fire` and `Wait(0.3)` are
//! visited but not on every tick. `Race` and `Wait(2.0)` are reached only
//! when `acquire_target` fails (rotated in by the closure). Over a full
//! 4-tick rotation, every leaf flashes at least once.

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
    loop {
        tick_n += 1;
        // Vary leaf statuses tick-by-tick so the visualizer shows visible
        // animation. The 4-tick rotation cycles through If branches and
        // Select's first-vs-second-child path.
        let outcome = bt.tick(&event, &mut |args, _bb| {
            let status = match (*args.action, tick_n % 4) {
                // If alternates on_success ↔ on_failure as low_hp toggles.
                ("low_hp", 1 | 2) => Status::Failure,
                // Force Select past its first child every other tick so
                // Race / Invert / Wait(2.0) get visited.
                ("acquire_target", 0 | 2) => Status::Failure,
                // Invert flips this — Failure → Inverter Success, vice versa.
                ("enemy_visible", 1) => Status::Success,
                ("enemy_visible", _) => Status::Failure,
                _ => Status::Success,
            };
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
