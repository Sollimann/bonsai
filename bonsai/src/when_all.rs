use crate::status::Status::*;
use crate::{event::UpdateEvent, state::State, ActionArgs, Status, RUNNING};

// `WhenAll` and `WhenAny` share same algorithm.
//
// `WhenAll` fails if any fails and succeeds when all succeeds.
// `WhenAny` succeeds if any succeeds and fails when all fails.
#[rustfmt::skip]
pub fn when_all<A, E, F, B>(
    any: bool,
    upd: Option<f64>,
    cursors: &mut [Option<State<A>>],
    e: &E,
    f: &mut F,
    blackboard: &mut B,
) -> (Status, f64)
where
    A: Clone,
    E: UpdateEvent,
    F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, f64),
{
    let (status, inv_status) = if any {
        // `WhenAny`
        (Status::Failure, Status::Success)
    } else {
        // `WhenAll`
        (Status::Success, Status::Failure)
    };
    // Get the least delta time left over.
    let mut min_dt = f64::MAX;
    // Count number of terminated events.
    let mut terminated = 0;
    for cur in cursors.iter_mut() {
        match *cur {
            None => {}
            Some(ref mut cur) => {
                match cur.tick(e, blackboard, f) {
                    (Running, _) => {
                        continue;
                    }
                    (s, new_dt) if s == inv_status => {
                        // Fail for `WhenAll`.
                        // Succeed for `WhenAny`.
                        return (inv_status, new_dt);
                    }
                    (s, new_dt) if s == status => {
                        min_dt = min_dt.min(new_dt);
                    }
                    _ => unreachable!(),
                }
            }
        }

        terminated += 1;
        *cur = None;
    }
    match terminated {
        // If there are no events, there is a whole 'dt' left.
        0 if cursors.is_empty() => (
            status,
            match upd {
                Some(dt) => dt,
                // Other kind of events happen instantly.
                _ => 0.0,
            },
        ),
        // If all events terminated, the least delta time is left.
        n if cursors.len() == n => (status, min_dt),
        _ => RUNNING,
    }
}
