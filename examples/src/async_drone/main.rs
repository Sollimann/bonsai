use bonsai_bt::{
    Event,
    Status::{self, Failure, Success},
    Timer, UpdateArgs, BT, RUNNING,
};
use futures::FutureExt;
use jobs::{collision_avoidance_task, landing_task, takeoff_task};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};

use crate::jobs::{fly_to_point_task, Point};
mod jobs;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum DroneAction {
    /// Collision avoidance
    AvoidOthers,
    /// Drone takeoff
    TakeOff,
    /// Land the drone
    Land,
    // /// Check battery
    CheckBattery,
    // /// Fly to point
    FlyToPoint(f32, f32, f32),
}

#[derive(Debug)]
pub struct DroneState {
    pub avoid_others: Option<Receiver<Status>>,
    pub takeoff: Option<Receiver<Status>>,
    pub land: Option<Receiver<Status>>,
    // pub check_battery: Option<Receiver<Status>>,
    pub fly_to_point: Option<Receiver<Status>>,
}

async fn drone_tick(
    timer: &mut Timer,
    drone_state: &mut DroneState,
    bt: &mut BT<DroneAction, HashMap<String, serde_json::Value>>,
) {
    // timer since bt was first invoked
    let _t = timer.duration_since_start();

    // have bt advance dt seconds into the future
    let dt = timer.get_dt();

    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt }.into();

    // update state of behaviosuccessr tree
    #[rustfmt::skip]
    bt.tick(&e,&mut |args: bonsai_bt::ActionArgs<Event, DroneAction>, _|
        match *args.action {
            DroneAction::AvoidOthers => {
                let avoid_state = &drone_state.avoid_others;
                if let Some(avoid_status) = avoid_state {
                    match avoid_status.recv() {
                        Ok(_) => RUNNING,
                        Err(_) => {
                            drone_state.avoid_others = None;
                            (Status::Failure, args.dt)
                        },
                    }
                } else {
                    println!("collision avoidance initialized");
                    let (tx, rx) = channel();
                    let (remote_job, handler) = collision_avoidance_task(tx).remote_handle();
                    handler.forget();
                    tokio::spawn(remote_job);
                    drone_state.avoid_others = Some(rx);
                    let receiver = drone_state.avoid_others.as_ref().unwrap();
                    let status = receiver.recv().unwrap();
                    (status, args.dt)
                }
            },
            DroneAction::TakeOff => {
                let takeoff_state = &drone_state.takeoff;
                if let Some(takeoff_status) = takeoff_state {
                    match takeoff_status.recv() {
                        Ok(status) => {
                            match status {
                                Success => {
                                    drone_state.takeoff = None;
                                    (Status::Success, args.dt)
                                },
                                Failure => {
                                    drone_state.takeoff = None;
                                    (Status::Failure, args.dt)
                                },
                                Status::Running => RUNNING,
                            }
                        },
                        Err(_) => {
                            drone_state.takeoff = None;
                            (Status::Failure, args.dt)
                        },
                    }
                } else {
                    println!("takeoff task initialized");
                    let (tx, rx) = channel();
                    let (remote_job, handler) = takeoff_task(tx).remote_handle();
                    handler.forget();
                    tokio::spawn(remote_job);
                    drone_state.takeoff = Some(rx);
                    let receiver = drone_state.takeoff.as_ref().unwrap();
                    let status = receiver.recv().unwrap();
                    (status, args.dt)
                }
            },
            DroneAction::Land => {
                let landing_state = &drone_state.land;
                if let Some(landing_status) = landing_state {
                    match landing_status.recv() {
                        Ok(status) => {
                            match status {
                                Success => {
                                    drone_state.land = None;
                                    (Status::Success, args.dt)
                                },
                                Failure => {
                                    drone_state.land = None;
                                    (Status::Failure, args.dt)
                                },
                                Status::Running => RUNNING,
                            }
                        },
                        Err(_) => {
                            drone_state.land = None;
                            (Status::Failure, args.dt)
                        },
                    }
                } else {
                    println!("landing task initialized");
                    let (tx, rx) = channel();
                    let (remote_job, handler) = landing_task(tx).remote_handle();
                    handler.forget();
                    tokio::spawn(remote_job);
                    drone_state.land = Some(rx);
                    let receiver = drone_state.land.as_ref().unwrap();
                    let status = receiver.recv().unwrap();
                    (status, args.dt)
                }
            },
            DroneAction::CheckBattery => {
                (Success, args.dt)
            },
            DroneAction::FlyToPoint(x, y, z) => {
                let flying_state = &drone_state.fly_to_point;
                if let Some(flying_status) = flying_state {
                    match flying_status.recv() {
                        Ok(status) => {
                            match status {
                                Success => {
                                    drone_state.fly_to_point = None;
                                    (Status::Success, args.dt)
                                },
                                Failure => {
                                    drone_state.fly_to_point = None;
                                    (Status::Failure, args.dt)
                                },
                                Status::Running => RUNNING,
                            }
                        },
                        Err(_) => {
                            drone_state.fly_to_point = None;
                            (Status::Failure, args.dt)
                        },
                    }
                } else {
                    println!("flying task initialized");
                    let (tx, rx) = channel();
                    let (remote_job, handler) = fly_to_point_task(Point::new(x, y, z), tx).remote_handle();
                    handler.forget();
                    tokio::spawn(remote_job);
                    drone_state.fly_to_point = Some(rx);
                    let receiver = drone_state.fly_to_point.as_ref().unwrap();
                    let status = receiver.recv().unwrap();
                    (status, args.dt)
                }
            },
        }
    ).unwrap();
}

#[tokio::main]
async fn main() {
    use bonsai_bt::{Action, Select, Sequence, While};
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    let avoid_others = Action(DroneAction::AvoidOthers);
    let takeoff = Action(DroneAction::TakeOff);
    let land = Action(DroneAction::Land);
    let is_battery_level_ok = Action(DroneAction::CheckBattery);

    let fly_if_healthy = Sequence(vec![
        is_battery_level_ok,
        Action(DroneAction::FlyToPoint(10.0, 10.0, 10.0)),
    ]);

    // if battery is low, then land
    let fly_to_dock = Action(DroneAction::FlyToPoint(0.0, 0.0, 0.0));
    let mission_with_fallback = Select(vec![fly_if_healthy, fly_to_dock]);

    let behavior = While(
        Box::new(avoid_others),
        // takeoff, do mission, land again
        vec![takeoff, mission_with_fallback, land],
    );

    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();
    let mut bt = BT::new(behavior, blackboard);
    let g = bt.get_graphviz();
    println!("{}", g);

    let mut timer = Timer::init_time();

    // initialize drone
    let mut drone_state = DroneState {
        avoid_others: None,
        takeoff: None,
        land: None,
        fly_to_point: None,
    };

    loop {
        sleep(Duration::new(0, 0.5e+9 as u32));
        drone_tick(&mut timer, &mut drone_state, &mut bt).await;
    }
}
