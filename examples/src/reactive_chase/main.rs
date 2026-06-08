//! Two reactive-composite demos in one binary.
//!
//! `ReactiveSequence`: chase the enemy while it is visible. When visibility
//! flips off mid-chase, the composite aborts the running `Chase` action on
//! the very next tick because the leading `EnemyVisible` condition is
//! re-evaluated from scratch. A regular [`bonsai_bt::Sequence`] would resume
//! the running child and keep chasing.
//!
//! `ReactiveSelect`: priority preemption. Tries `Attack` first; while the
//! enemy is out of range, attack fails and the composite falls through to
//! `Chase`. Once the enemy enters range, the next tick aborts the chase and
//! runs `Attack` instead.

use bonsai_bt::{Action, ActionArgs, Event, ReactiveSelect, ReactiveSequence, Status, UpdateArgs, BT};

#[derive(Clone, Copy, Debug)]
enum Act {
    EnemyVisible,
    Chase,
    Attack,
}

#[derive(Default)]
struct World {
    visible: bool,
    in_range: bool,
    chase_progress: u32,
    attack_progress: u32,
}

fn run_chase_demo() {
    println!("=== ReactiveSequence: chase the enemy while it stays visible ===");
    println!("Setup: enemy starts visible. At tick 3 it vanishes from sight.\n");

    let tree = ReactiveSequence(vec![Action(Act::EnemyVisible), Action(Act::Chase)]);
    let mut bt = BT::new(
        tree,
        World {
            visible: true,
            ..World::default()
        },
    );

    for t in 0..6 {
        if t == 3 {
            bt.blackboard_mut().visible = false;
            println!("  >>> enemy vanishes; leading condition will now fail <<<");
        }

        let e: Event = UpdateArgs { dt: 0.1 }.into();
        let _ = bt.tick(
            &e,
            &mut |args: ActionArgs<Event, Act>, bb: &mut World| match *args.action {
                Act::EnemyVisible => {
                    if bb.visible {
                        (Status::Success, args.dt)
                    } else {
                        (Status::Failure, args.dt)
                    }
                }
                Act::Chase => {
                    bb.chase_progress += 1;
                    (Status::Running, 0.0)
                }
                _ => unreachable!(),
            },
        );

        let bb = bt.blackboard();
        let state = if bb.visible { "visible" } else { "hidden " };
        let activity = if bb.visible {
            "chasing"
        } else {
            "idle (chase aborted; steps frozen)"
        };
        println!(
            "tick {t}: enemy {state}  ->  {activity:36}  chase_steps={}",
            bb.chase_progress
        );

        if bt.is_finished() {
            bt.reset_bt();
        }
    }
}

fn run_priority_demo() {
    println!("\n=== ReactiveSelect: prefer attack when in range; otherwise chase ===");
    println!("Setup: enemy starts out of range. At tick 3 it closes to melee.\n");

    let tree = ReactiveSelect(vec![Action(Act::Attack), Action(Act::Chase)]);
    let mut bt = BT::new(tree, World::default());

    for t in 0..6 {
        if t == 3 {
            bt.blackboard_mut().in_range = true;
            println!("  >>> enemy enters range; Attack now succeeds and preempts Chase <<<");
        }

        let e: Event = UpdateArgs { dt: 0.1 }.into();
        let _ = bt.tick(
            &e,
            &mut |args: ActionArgs<Event, Act>, bb: &mut World| match *args.action {
                Act::Attack => {
                    if bb.in_range {
                        bb.attack_progress += 1;
                        (Status::Running, 0.0)
                    } else {
                        (Status::Failure, args.dt)
                    }
                }
                Act::Chase => {
                    bb.chase_progress += 1;
                    (Status::Running, 0.0)
                }
                _ => unreachable!(),
            },
        );

        let bb = bt.blackboard();
        let state = if bb.in_range { "in range    " } else { "out of range" };
        let activity = if bb.in_range {
            "attacking"
        } else {
            "attack fails -> falling back to chase"
        };
        println!(
            "tick {t}: enemy {state}  ->  {activity:38}  chase={}, attack={}",
            bb.chase_progress, bb.attack_progress
        );

        if bt.is_finished() {
            bt.reset_bt();
        }
    }
}

fn main() {
    run_chase_demo();
    run_priority_demo();
}
