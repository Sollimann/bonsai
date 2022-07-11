use std::{collections::HashMap, thread::sleep, time::Duration};

use bonsai::{
    Behavior::{Action, Select, Sequence},
    Event, Timer, UpdateArgs, BT,
};

type Damage = u32;
type Distance = f64;
type Time = f64;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
enum EnemyNPC {
    /// Run
    Run,
    /// Cover
    GetInCover,
    /// Blind fire for some time
    BlindFire(Time, Damage),
    /// When player is close -> melee attack
    MeleeAttack(Damage),
    /// When player is far away -> fire weapon
    FireWeapon(Damage),
    /// Return true if player within striking distance
    PlayerIsClose(Distance),
}

fn game_tick(timer: &mut Timer, bt: &mut BT<EnemyNPC, String, serde_json::Value>) {
    // time since last invovation of bt
    let dt = timer.get_dt();

    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt }.into();

    #[rustfmt::skip]
    bt.state.tick(&e,&mut |args: bonsai::ActionArgs<Event, EnemyNPC>| {
        match *args.action {
            EnemyNPC::Run => todo!(),
            EnemyNPC::GetInCover => todo!(),
            EnemyNPC::BlindFire(_, _) => todo!(),
            EnemyNPC::MeleeAttack(_) => todo!(),
            EnemyNPC::FireWeapon(_) => todo!(),
            EnemyNPC::PlayerIsClose(_) => todo!(),
        }
    });
}

fn main() {
    use crate::EnemyNPC::{BlindFire, FireWeapon, GetInCover, MeleeAttack, Run};

    // define blackboard (even though we're not using it)
    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();

    // create ai behavior
    let run = Action(Run);
    let get_in_cover = Action(GetInCover);

    let run_cover = Sequence(vec![run, get_in_cover, Action(BlindFire(2.0, 50))]);
    let player_close = Select(vec![Action(MeleeAttack(100)), Action(FireWeapon(50))]);
    let under_fire_behavior = Select(vec![run_cover, player_close]);

    let bt_serialized = serde_json::to_string_pretty(&under_fire_behavior).unwrap();
    println!("creating bt: \n {} \n", bt_serialized);
    let mut bt = BT::new(under_fire_behavior, blackboard);

    // create a monotonic timer
    let mut timer = Timer::init_time();

    loop {
        // decide bt frequency by sleeping the loop
        sleep(Duration::new(0, 0.1e+9 as u32));

        // tick the bt
        game_tick(&mut timer, &mut bt);
    }
}
