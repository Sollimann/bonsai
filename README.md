<h1 align="center" style="font-family:Papyrus; font-size:4em;"> Bonsai 盆栽 </h1>
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/gifs/bonsai.gif" width="350" ">
</p>

<p align="center">
    <em>Rust implementation of Behavior Trees</em>
</p>

[![codecov](https://codecov.io/gh/Sollimann/bonsai/branch/main/graph/badge.svg?token=JX8JBPWORV)](https://codecov.io/gh/Sollimann/bonsai)
<!-- [![version](https://img.shields.io/badge/version-1.0.0-blue)](https://GitHub.com/Sollimann/CleanIt/releases/) -->
[![Build Status](https://github.com/Sollimann/bonsai/workflows/rust-cd/badge.svg)](https://github.com/Sollimann/bonsai/actions)
[![Bonsai crate](https://img.shields.io/crates/v/bonsai-bt.svg)](https://crates.io/crates/bonsai-bt)
[![minimum rustc 1.56](https://img.shields.io/badge/rustc-1.56+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Docs](https://docs.rs/bonsai-bt/badge.svg)](https://docs.rs/bonsai-bt)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Sollimann/bonsai/graphs/commit-activity)
[![GitHub pull-requests](https://img.shields.io/github/issues-pr/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
[![GitHub pull-requests closed](https://img.shields.io/github/issues-pr-closed/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
![ViewCount](https://views.whatilearened.today/views/github/Sollimann/bonsai.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Contents

* [Quick intro to Behavior Trees](https://www.youtube.com/watch?v=KeShMInMjro)
* [Concepts](docs/concepts/README.md)
* [Examples](examples/README.md)
* [Development Guide](DEVELOPMENT.md)
* [Kanban Board](https://github.com/Sollimann/b3/projects/1)

## Using Bonsai
Bonsai is available on crates.io. The recommended way to use it is to add a line into your Cargo.toml such as:

```toml
[dependencies]
bonsai-bt = "*"
```

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
- If `A` succeeds, return failure (and vice-versa): `Invert(A)`
- Do `B` repeatedly while `A` runs: `While(A, [B])`
- Do `A`, `B` forever: `While(WaitForever, [A, B])`
- Wait for both `A` and `B` to complete: `WhenAll([A, B])`
- Wait for either `A` or `B` to complete: `WhenAny([A, B])`
- Wait for either `A` or `B` to complete: `WhenAny([A, B])`

See the `Behavior` enum for more information.

## Example of use

This is a enemy NPC (non-player-character) behavior mock-up which decides if the AI should shoot while running for nearby cover, rush in to attack the player up close or stand its ground while firing at the player.

#### Tree vizualization
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/npc_bt.png" width="700" ">
</p>

#### Implementation
```rust
use std::{collections::HashMap, thread::sleep, time::Duration};

use bonsai_bt::{
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
    // how much time should the bt advance into the future
    let dt = timer.get_dt();

    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt }.into();

    #[rustfmt::skip]
    bt.state.tick(&e,&mut |args: bonsai_bt::ActionArgs<Event, EnemyNPC>| {
        match *args.action {
            EnemyNPC::Run => {
              Enemy::run_away_from_player(); // you must implement these methods
              (bonsai_bt::Running, 0.0)
            },
            EnemyNPC::GetInCover => {
              let in_cover: Bool = Enemy::get_in_cover();
              if in_cover {
                (bonsai_bt::Success, dt)
              } else {
                (bonsai_bt::Running, 0.0)
              }
            },
            EnemyNPC::BlindFire(damage) => {
              let has_ammo: Bool = Enemy::has_ammo();
              if has_ammo {
                Enemy::shoot_in_direction();
                (bonsai_bt::Success, dt)
              } else {
                (bonsai_bt::Failure, dt)
              }
            },
            EnemyNPC::MeleeAttack(dist, damage) => {
              let player = Player::get_player();
              let pos = Enemy::get_pos();
              let diff = sub(*pos, player.pos);
              if len(diff) < dist {
                  let &mut player_health = Player::get_health();
                  *player_health = Player::decrease_health(damage);
                  (bonsai_bt::Success, dt)
              } else {
                  (bonsai_bt::Failure, dt)
              }
            },
            EnemyNPC::FireWeapon(damage) => {
              let has_ammo: Bool = Enemy::has_ammo();
              if has_ammo {
                Enemy::shoot_at_player();
                (bonsai_bt::Success, dt)
              } else {
                (bonsai_bt::Failure, dt)
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
