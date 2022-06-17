use crate::{ActionArgs, State, Status, RUNNING};

// `WhenAll` and `WhenAny` share same algorithm.
//
// `WhenAll` fails if any fails and succeeds when all succeeds.
// `WhenAny` succeeds if any succeeds and fails when all fails.
#[rustfmt::skip]
pub fn when_all<A, S, F>(
    any: bool,
    upd: Option<f64>,
    cursors: &mut Vec<Option<State<A, S>>>,
    f: &mut F,
) -> (Status, f64)
where
    A: Clone,
    F: FnMut(ActionArgs<A, S>) -> (Status, f64),
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
                match cur.event(upd, f) {
                    (Status::Running, _) => {
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
