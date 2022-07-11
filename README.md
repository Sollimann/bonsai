<h1 align="center" style="font-family:Papyrus; font-size:4em;"> Bonsai 盆栽 </h1>
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/gifs/bonsai.gif" width="350" ">
</p>

<p align="center">
    <em>Rust implementation of Behavior Trees</em>
</p>

<!-- [![codecov](https://codecov.io/gh/Sollimann/CleanIt/branch/main/graph/badge.svg?token=EY3JRZN71M)](https://codecov.io/gh/Sollimann/CleanIt) -->
<!-- [![version](https://img.shields.io/badge/version-1.0.0-blue)](https://GitHub.com/Sollimann/CleanIt/releases/) -->
[![Build Status](https://github.com/Sollimann/bonsai/workflows/rust-ci/badge.svg)](https://github.com/Sollimann/bonsai/actions)
[![minimum rustc 1.60](https://img.shields.io/badge/rustc-1.60+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Sollimann/bonsai/graphs/commit-activity)
[![GitHub pull-requests](https://img.shields.io/github/issues-pr/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
[![GitHub pull-requests closed](https://img.shields.io/github/issues-pr-closed/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
![ViewCount](https://views.whatilearened.today/views/github/Sollimann/bonsai.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Contents

* [Concepts](docs/concepts/README.md)
* [Development Guide](DEVELOPMENT.md)
* [Kanban Board](https://github.com/Sollimann/b3/projects/1)

## What is a Behavior Tree?

A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.

### How to use a Behavior tree?

An AI behavior tree is a very generic way of organizing interactive logic.
It has built-in semantics for processes that signals `Running`, `Success` or
`Failure`.

For example, if you have a state `A` and a state `B`:

- Move from state `A` to state `B` if `A` succeeds: `Sequence([A, B])`
- Try `A` first and then try `B` if `A` fails: `Select([A, B])`
- If `condition` succeedes do `A`, else do `B` : `If(condition, A, B)`
- Do `B` repeatedly while `A` runs: `While(A, [B])`
- Do `A`, `B` forever: `While(WaitForever, [A, B])`
- Wait for both `A` and `B` to complete: `WhenAll([A, B])`
- Wait for either `A` or `B` to complete: `WhenAny([A, B])`

See the `Behavior` enum for more information.

## Example of use

This is a enemy NPC (non-player-character) behavior mock-up which decides if the AI should shoot will running for nearby cover, rush in to attack the player up close or stand its ground while firing at the player.

#### Tree vizualization
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/readme-example/docs/resources/images/npc_bt.png" width="600" ">
</p>

#### Implementation
```rust
use std::{collections::HashMap, thread::sleep, time::Duration};

use bonsai::{
    Behavior::{Action, Select, Sequence},
    Event, Status, Running, Timer, UpdateArgs, BT,
};

type Damage = u32;
type Distance = f64;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
enum EnemyNPC {
    /// Run
    Run,
    /// Cover
    GetInCover,
    /// Blind fire
    BlindFire(Damage),
    /// When player is close -> melee attack
    ///
    /// distance [m], damage
    MeleeAttack(Distance, Damage),
    /// When player is far away -> fire weapon
    FireWeapon(Damage),
}

use game::{Enemy, Player}; // fictive game imports

fn game_tick(timer: &mut Timer, bt: &mut BT<EnemyNPC, String, serde_json::Value>) {
    // time since last invovation of bt
    let dt = timer.get_dt();

    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt }.into();

    #[rustfmt::skip]
    bt.state.tick(&e,&mut |args: bonsai::ActionArgs<Event, EnemyNPC>| {
        match *args.action {
            EnemyNPC::Run => {
              Enemy::run_away_from_player(); // you must implement these methods
              (bonsai::Running, 0.0)
            },
            EnemyNPC::GetInCover => {
              let in_cover: Bool = Enemy::get_in_cover();
              if in_cover {
                (bonsai::Success, dt)
              } else {
                (bonsai::Running, 0.0)
              }
            },
            EnemyNPC::BlindFire(damage) => {
              let has_ammo: Bool = Enemy::has_ammo();
              if has_ammo {
                Enemy::shoot_in_direction();
                (bonsai::Success, dt)
              } else {
                (bonsai::Failure, dt)
              }
            },
            EnemyNPC::MeleeAttack(dist, damage) => {
              let player = Player::get_player();
              let pos = Enemy::get_pos();
              let diff = sub(*pos, player.pos);
              if len(diff) < dist {
                  let &mut player_health = Player::get_health();
                  *player_health = Player::decrease_health(damage);
                  (bonsai::Success, dt)
              } else {
                  (bonsai::Failure, dt)
              }
            },
            EnemyNPC::FireWeapon(damage) => {
              let has_ammo: Bool = Enemy::has_ammo();
              if has_ammo {
                Enemy::shoot_at_player();
                (bonsai::Success, dt)
              } else {
                (bonsai::Failure, dt)
              }
            },
        }
    });
}

fn main() {
    use crate::EnemyNPC::{BlindFire, FireWeapon, GetInCover, MeleeAttack, Run};

    // define blackboard (even though we're not using it)
    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();

    // create ai behavior
    let run = Action(Run);
    let cover = Action(GetInCover);
    let run_for_five_secs = While(Box::new(Wait(5.0)), vec![run]);
    let run_and_shot = While(Box::new(run_for_five_secs), vec![Action(BlindFire(50))]);
    let run_cover = Sequence(vec![run_and_shot, cover]);

    let player_close = Select(vec![Action(MeleeAttack(1.0, 100)), Action(FireWeapon(50))]);
    let under_attack_behavior = Select(vec![run_cover, player_close]);

    let bt_serialized = serde_json::to_string_pretty(&under_attack_behavior).unwrap();
    println!("creating bt: \n {} \n", bt_serialized);
    let mut bt = BT::new(under_attack_behavior, blackboard);

    // create a monotonic timer
    let mut timer = Timer::init_time();

    loop {
        // decide bt frequency by sleeping the loop
        sleep(Duration::new(0, 0.1e+9 as u32));

        // tick the bt
        game_tick(&mut timer, &mut bt);
    }
}
```
