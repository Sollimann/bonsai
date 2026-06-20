use bonsai_bt::{
    Behavior::{Action, Sequence, Wait, WaitForever, WhenAny, While},
    BT,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Damage = f64;
type Distance = f64;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
enum AttackDrone {
    /// Circles forever around target pos.
    Circling,
    /// Waits until player is within distance.
    PlayerWithinDistance(Distance),
    /// Fly toward player.
    FlyTowardPlayer,
    /// Waits until player is far away from target.
    PlayerFarAwayFromTarget(Distance),
    /// Makes player loose more blood.
    AttackPlayer(Damage),
}

fn main() {
    // define blackboard (even though we're not using it)
    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();

    // create ai behavior
    let circling = Action(AttackDrone::Circling);
    let circle_until_player_within_distance = Sequence(vec![
        While(Box::new(Wait(5.0)), vec![circling.clone()]),
        While(
            Box::new(Action(AttackDrone::PlayerWithinDistance(50.0))),
            vec![circling],
        ),
    ]);
    let give_up_or_attack = WhenAny(vec![
        Action(AttackDrone::PlayerFarAwayFromTarget(100.0)),
        Sequence(vec![
            Action(AttackDrone::PlayerWithinDistance(10.0)),
            Action(AttackDrone::AttackPlayer(0.1)),
        ]),
    ]);
    let attack_attempt = While(Box::new(give_up_or_attack), vec![Action(AttackDrone::FlyTowardPlayer)]);
    let behavior = While(
        Box::new(WaitForever),
        vec![circle_until_player_within_distance, attack_attempt],
    );

    // serialize BT to json
    let bt_serialized = serde_json::to_string_pretty(&behavior).unwrap();
    println!("creating bt: \n {} \n", bt_serialized);

    let mut bt = BT::new(behavior, blackboard);

    // produce a string DiGraph compatible with graphviz
    // paste the contents in graphviz, e.g: https://dreampuf.github.io/GraphvizOnline/#
    let g = bt.get_graphviz();
    println!("{}", g);
}
