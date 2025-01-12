use crate::status::Status::*;
use crate::{event::UpdateEvent, ActionArgs, Behavior, State, Status, RUNNING};

pub struct SequenceArgs<'a, A, E, F, B> {
    pub select: bool,
    pub upd: Option<f64>,
    pub seq: &'a [Behavior<A>],
    pub i: &'a mut usize,
    pub cursor: &'a mut Box<State<A>>,
    pub e: &'a E,
    pub blackboard: &'a mut B,
    pub f: &'a mut F,
}

// `Sequence` and `Select` share same algorithm.
//
// `Sequence` fails if any fails and succeeds when all succeeds.
// `Select` succeeds if any succeeds and fails when all fails.
pub fn sequence<A, E, F, B>(args: SequenceArgs<A, E, F, B>) -> (Status, f64)
where
    A: Clone,
    E: UpdateEvent,
    F: FnMut(&A, &mut B, ActionArgs<E>) -> (Status, f64),
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
    } = args;

    let (status, inv_status) = if select {
        // `Select`
        (Status::Failure, Status::Success)
    } else {
        // `Sequence`
        (Status::Success, Status::Failure)
    };
    let mut remaining_dt = upd.unwrap_or(0.0);
    let mut remaining_e;
    while *i < seq.len() {
        match cursor.tick(
            match upd {
                Some(_) => {
                    remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                    &remaining_e
                }
                _ => e,
            },
            blackboard,
            f,
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
