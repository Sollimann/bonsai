use crate::status::Status::*;
use crate::telemetry::{NodeMeta, Tracer};
use crate::Float;
use crate::{event::UpdateEvent, state::State, ActionArgs, Status, RUNNING};

// `WhenAll` and `WhenAny` share same algorithm.
//
// `WhenAll` fails if any fails and succeeds when all succeeds.
// `WhenAny` succeeds if any succeeds and fails when all fails.
#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
pub fn when_all<A, E, F, B, T>(
    any: bool,
    upd: Option<Float>,
    cursors: &mut [Option<State<A>>],
    e: &E,
    f: &mut F,
    blackboard: &mut B,
    parent_id: usize,
    metas: &[NodeMeta],
    tracer: &mut T,
) -> (Status, Float)
where
    A: Clone,
    E: UpdateEvent,
    F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    T: Tracer,
{
    let (status, inv_status) = if any {
        // `WhenAny`
        (Status::Failure, Status::Success)
    } else {
        // `WhenAll`
        (Status::Success, Status::Failure)
    };
    // Get the least delta time left over.
    let mut min_dt = Float::MAX;
    // Count number of terminated events.
    let mut terminated = 0;
    let mut child_id = if T::IS_RECORDING { parent_id + 1 } else { 0 };
    for cur in cursors.iter_mut() {
        let this_id = child_id;
        if T::IS_RECORDING {
            child_id += metas[this_id].subtree_size;
        }
        match *cur {
            None => {}
            Some(ref mut cur) => {
                match cur.tick(this_id, metas, e, blackboard, f, tracer) {
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
    #[allow(clippy::manual_unwrap_or)]
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
