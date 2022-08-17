use std::collections::HashMap;

use bonsai_bt::{
    Behavior::{Action, Select, Sequence, Wait, While},
    BT,
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

fn main() {
    use crate::EnemyNPC::{BlindFire, FireWeapon, GetInCover, MeleeAttack, Run};

    // define blackboard (even though we're not using it)
    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();

    // create ai behavior
    let run = Action(Run);
    let cover = Action(GetInCover);
    let run_for_five_secs = While(Box::new(Wait(5.0)), vec![run]);
    let run_and_shoot = While(Box::new(run_for_five_secs), vec![Action(BlindFire(50))]);
    let run_cover = Sequence(vec![run_and_shoot, cover]);

    let player_close = Select(vec![Action(MeleeAttack(1.0, 100)), Action(FireWeapon(50))]);
    let under_attack_behavior = Select(vec![run_cover, player_close]);

    // serialize BT to json
    let bt_serialized = serde_json::to_string_pretty(&under_attack_behavior).unwrap();
    println!("creating bt: \n {} \n", bt_serialized);

    let mut bt = BT::new(under_attack_behavior, blackboard);

    // produce a string DiGraph compatible with graphviz
    // paste the contents in graphviz, e.g: https://dreampuf.github.io/GraphvizOnline/#
    let g = bt.get_graphviz();
    println!("{}", g);
}
