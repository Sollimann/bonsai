use bonsai_bt::Behavior::Wait;
use bonsai_bt::{
    Behavior::Action, Behavior::Race, Behavior::Sequence, Event, Float, Status, Timer, UpdateArgs, BT, RUNNING,
};
use futures::FutureExt;
use rand::Rng;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::thread::sleep;
use std::time::Duration;
use tokio::time::sleep as async_sleep;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum MissionAction {
    /// The main job that finishes after a random delay
    DoWork,
    /// Hard deadline that fires after a fixed delay
    OnTimeout,
}

pub struct MissionState {
    pub work: Option<Receiver<Status>>,
}

/// Simulates a unit of work whose duration is random.
/// Sometimes it finishes before the timeout and sometimes it doesn't.
async fn do_work_task(tx: std::sync::mpsc::Sender<Status>) {
    let work_ms: u64 = rand::thread_rng().gen_range(200..=1200);
    println!("[do_work] started.");

    let step = Duration::from_millis(100);
    let mut elapsed = 0u64;
    while elapsed < work_ms {
        if tx.send(Status::Running).is_err() {
            println!("[do_work] preempted by timeout, stopping.");
            return;
        }
        async_sleep(step).await;
        elapsed += step.as_millis() as u64;
    }

    println!("[do_work] finished after {elapsed} ms");
    let _ = tx.send(Status::Success);
}

async fn tick(
    timer: &mut Timer,
    state: &mut MissionState,
    bt: &mut BT<MissionAction, HashMap<String, serde_json::Value>>,
) -> std::option::Option<(Status, Float)> {
    let dt: Float = timer.get_dt();
    let e: Event = UpdateArgs { dt }.into();

    bt.tick(
        &e,
        &mut |args: bonsai_bt::ActionArgs<Event, MissionAction>, _| match *args.action {
            MissionAction::DoWork => {
                if let Some(rx) = &state.work {
                    match rx.recv() {
                        Ok(Status::Running) => RUNNING,
                        Ok(Status::Success) => {
                            state.work = None;
                            (Status::Success, args.dt)
                        }
                        Ok(Status::Failure) | Err(_) => {
                            state.work = None;
                            (Status::Failure, args.dt)
                        }
                    }
                } else {
                    let (tx, rx) = channel();
                    let (job, handle) = do_work_task(tx).remote_handle();
                    handle.forget();
                    tokio::spawn(job);
                    state.work = Some(rx);
                    match state.work.as_ref().unwrap().recv().unwrap() {
                        Status::Running => RUNNING,
                        s => (s, args.dt),
                    }
                }
            }

            MissionAction::OnTimeout => {
                eprintln!("do_work timed out!");
                (Status::Failure, args.dt)
            }
        },
    )
}

#[tokio::main]
async fn main() {
    const TIMEOUT_S: Float = 0.6;

    let behavior = Sequence(vec![Race(vec![
        Action(MissionAction::DoWork),
        Sequence(vec![Wait(TIMEOUT_S), Action(MissionAction::OnTimeout)]),
    ])]);

    let mut bt = BT::new(behavior, HashMap::new());
    let mut timer = Timer::init_time();
    let mut state = MissionState { work: None };

    loop {
        sleep(Duration::from_millis(50));
        if tick(&mut timer, &mut state, &mut bt).await.is_none() {
            break;
        }
    }
}
