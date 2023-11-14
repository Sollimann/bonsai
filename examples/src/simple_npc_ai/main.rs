use std::collections::HashMap;

use bonsai_bt::Behavior::RepeatSequence;
use bonsai_bt::{Behavior::Action, Event, Failure, Running, Status, Success, UpdateArgs, BT};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub enum EnemyNPC {
    Run,
    Jump,
    Shoot,
    HasActionPointsLeft,
}

fn game_tick(bt: &mut BT<EnemyNPC, (), ()>, state: &mut EnemyNPCState) -> Status {
    let e: Event = UpdateArgs { dt: 0.0 }.into();

    #[rustfmt::skip]
    let status = bt.state.tick(&e,&mut |args: bonsai_bt::ActionArgs<Event, EnemyNPC>| {
        match *args.action {
            EnemyNPC::Run => {
                state.perform_action("run");
                return (Success, 0.0)
            },
            EnemyNPC::HasActionPointsLeft => {
                print!("Is NPC tired... ");
                if state.action_points == 0 {
                    println!("yes!");
                    return (Failure, 0.0);
                }
                else {
                    println!("no! Action points: {}", state.action_points );
                    return(Running, 0.0)
                }
            }
            EnemyNPC::Jump => {
                state.perform_action("jump");
                return(Success, 0.0)
            }
            EnemyNPC::Shoot => {
                state.perform_action("shoot");
                return(Success, 0.0)
            }
        }
    });

    // return status:
    status.0
}

struct EnemyNPCState {
    pub action_points: usize,
    pub max_action_points: usize,
}
impl EnemyNPCState {
    fn consume_action_point(&mut self) {
        self.action_points = self.action_points.checked_sub(1).unwrap_or(0);
    }
    fn rest(&mut self) {
        self.action_points = self.max_action_points;
    }

    fn perform_action(&mut self, action: &str) {
        if self.action_points > 0 {
            self.consume_action_point();
            println!("Performing action: {}. Action points: {}", action, self.action_points);
        } else {
            println!("Cannot perform action: {}. Not enough action points", action);
        }
    }
}

fn main() {
    // define blackboard (even though we're not using it)
    let blackboard: HashMap<(), ()> = HashMap::new();

    let npc_ai = RepeatSequence(
        Box::new(Action(EnemyNPC::HasActionPointsLeft)),
        vec![Action(EnemyNPC::Run), Action(EnemyNPC::Jump), Action(EnemyNPC::Shoot)],
    );
    let mut bt = BT::new(npc_ai, blackboard);

    let mut npc_state = EnemyNPCState {
        action_points: 10,
        max_action_points: 10,
    };

    loop {
        match game_tick(&mut bt, &mut npc_state) {
            Success => {}
            Failure => {
                break;
            }
            Running => {}
        }
    }
}
