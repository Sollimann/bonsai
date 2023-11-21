use std::collections::HashMap;

use bonsai_bt::Behavior::RepeatSequence;
use bonsai_bt::{Behavior::Action, Event, Failure, Running, Status, Success, UpdateArgs, BT, While, Sequence};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub enum EnemyNPC {
    Run,
    Shoot,
    HasActionPointsLeft,
    Rest,
    Die,
    IsDead,
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
                if state.action_points == 0 {
                    println!("NPC does not have actions points left... ");
                    return (Success, 0.0);
                }
                else {
                    println!("NPC has action points: {}", state.action_points );
                    return(Running, 0.0)
                }
            }
            EnemyNPC::Shoot => {
                state.perform_action("shoot");
                return(Success, 0.0)
            }
            EnemyNPC::Rest => {
                if state.fully_rested() {
                    return (Success, 0.0)
                }
                state.rest();
                return (Running, 0.0)
            }
            EnemyNPC::Die => {
                state.die();
                return (Success, 0.0);
            }
            EnemyNPC::IsDead => {
                if state.is_alive() {
                    return (Running, 0.0);
                }
                return (Success, 0.0);
            }
        }
    });

    // return status:
    status.0
}

struct EnemyNPCState {
    pub action_points: usize,
    pub max_action_points: usize,
    pub alive: bool,
}
impl EnemyNPCState {
    fn consume_action_point(&mut self) {
        self.action_points = self.action_points.checked_sub(1).unwrap_or(0);
    }
    fn rest(&mut self) {
        self.action_points = (self.action_points + 1).min(self.max_action_points);
        println!("Rested for a while... Action points: {}", self.action_points);
    }
    fn die(&mut self) {
        println!("NPC died...");
        self.alive = false
    }
    fn is_alive(&self) -> bool {
        if self.alive {
            println!("NPC is alive...");
        }
        else {
            println!("NPC is dead...");
        }
        self.alive
    }
    fn fully_rested(&self) -> bool {
        self.action_points == self.max_action_points
    }

    fn perform_action(&mut self, action: &str) {
        if self.action_points > 0 {
            self.consume_action_point();
            println!("Performing action: {}. Action points: {}", action, self.action_points);
        } else {
            println!("Cannot perform action: {}. Not enough action points.", action);
        }
    }
}

/// Demonstrates a usage of [RepeatSequence] behavior with
/// a simple NPC simulation.
///
/// The NPC AI first enters a higher [RepeatSequence] that
/// checks if the NPC is dead, then it succeeds to inner [RepeatSequence]
/// where the NPC performs actions until it is determined that
/// no action points are left to consume. Then the AI control flow returns
/// to the previous higher sequence where the executions continues and the NPC rests
/// and regains its actions points. After that the NPC is killed and it is once again
/// checked if the NPC is alive. Then the program quits.
///
/// Timeline of execution in more detail:
///
/// 1. check if the NPC is dead (no)
/// 2. execute "run and shoot" subprogram
/// 3. check if action points are available (yes)
/// 4. run
/// 5. shoot
/// 6. check if action points are available (yes)
/// 7. run
/// 8. shoot (notice that we don't have action points
///           here but we try anyway and move on the sequence)
/// 9. check if action points are available (no)
/// 10. exit the subprogram
/// 11. rest and regain action points
///         (this action returns [Running] until fully rested
///          so control flow is returned to main loop)
/// 12. kill the NPC
/// 13. check if the NPC is dead (yes)
/// 14. quit
///
///
///
///
fn main() {
    // define blackboard (even though we're not using it)
    let blackboard: HashMap<(), ()> = HashMap::new();

    let run_and_shoot_ai = RepeatSequence(
        Box::new(Action(EnemyNPC::HasActionPointsLeft)),
        vec![Action(EnemyNPC::Run), Action(EnemyNPC::Shoot)],
    );
    let top_ai = RepeatSequence(
        Box::new(Action(EnemyNPC::IsDead)),
        vec![run_and_shoot_ai.clone(), Action(EnemyNPC::Rest), Action(EnemyNPC::Die)],
    );
    let mut bt = BT::new(top_ai, blackboard);

    let print_graph = false;
    if print_graph {
        println!("{}", bt.get_graphviz());
    }

    let max_actions = 3;
    let mut npc_state = EnemyNPCState {
        action_points: max_actions,
        max_action_points: max_actions,
        alive: true,
    };


    loop {
        println!("reached main loop...");
        match game_tick(&mut bt, &mut npc_state) {
            Success |
            Failure => {
                break;
            }
            Running => {}
        }
    }
}
