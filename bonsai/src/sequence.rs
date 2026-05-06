use crate::status::Status::*;
use crate::telemetry::{first_child_id, next_sibling_id, NodeMeta, Tracer};
use crate::Float;
use crate::{event::UpdateEvent, state::State, ActionArgs, Behavior, Status, RUNNING};

pub struct SequenceArgs<'a, A, E, F, B, T> {
    pub select: bool,
    pub upd: Option<Float>,
    pub seq: &'a [Behavior<A>],
    pub i: &'a mut usize,
    pub cursor: &'a mut Box<State<A>>,
    pub e: &'a E,
    pub blackboard: &'a mut B,
    pub f: &'a mut F,
    pub parent_id: usize,
    pub metas: &'a [NodeMeta],
    pub tracer: &'a mut T,
}

// `Sequence` and `Select` share same algorithm.
//
// `Sequence` fails if any fails and succeeds when all succeeds.
// `Select` succeeds if any succeeds and fails when all fails.
pub fn sequence<A, E, F, B, T>(args: SequenceArgs<A, E, F, B, T>) -> (Status, Float)
where
    A: Clone,
    E: UpdateEvent,
    F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    T: Tracer,
{
    let SequenceArgs {
        select,
        upd,
        seq,
        i,
        cursor,
        e,
        blackboard,
        f,
        parent_id,
        metas,
        tracer,
    } = args;

    let (status, inv_status) = if select {
        // `Select`
        (Status::Failure, Status::Success)
    } else {
        // `Sequence`
        (Status::Success, Status::Failure)
    };
    let mut child_id = first_child_id::<T>(parent_id);
    if T::IS_RECORDING {
        for _ in 0..*i {
            child_id = next_sibling_id::<T>(metas, child_id);
        }
    }
    let mut remaining_dt = upd.unwrap_or(0.0);
    let mut remaining_e;
    while *i < seq.len() {
        match cursor.tick(
            child_id,
            metas,
            match upd {
                Some(_) => {
                    remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                    &remaining_e
                }
                _ => e,
            },
            blackboard,
            f,
            tracer,
        ) {
            (Running, _) => {
                break;
            }
            (s, new_dt) if s == inv_status => {
                return (inv_status, new_dt);
            }
            (s, new_dt) if s == status => {
                remaining_dt = match upd {
                    // Change update event with remaining delta time.
                    Some(_) => new_dt,
                    // Other events are 'consumed' and not passed to next.
                    // If this is the last event, then the sequence succeeded.
                    _ => {
                        if *i == seq.len() - 1 {
                            return (status, new_dt);
                        } else {
                            *i += 1;
                            // Create a new cursor for next event.
                            // Use the same pointer to avoid allocation.
                            **cursor = State::new(seq[*i].clone());
                            return RUNNING;
                        }
                    }
                }
            }
            _ => unreachable!(),
        };
        *i += 1;
        if T::IS_RECORDING {
            child_id = next_sibling_id::<T>(metas, child_id);
        }
        // If end of sequence,
        // return the 'dt' that is left.
        if *i >= seq.len() {
            return (status, remaining_dt);
        }
        // Create a new cursor for next event.
        // Use the same pointer to avoid allocation.
        **cursor = State::new(seq[*i].clone());
    }
    RUNNING
}
