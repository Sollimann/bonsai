use crate::{ActionArgs, Behavior, State, Status, RUNNING};

// `Sequence` and `Select` share same algorithm.
//
// `Sequence` fails if any fails and succeeds when all succeeds.
// `Select` succeeds if any succeeds and fails when all fails.
pub fn sequence<A, S, F>(
    select: bool,
    upd: Option<f64>,
    seq: &[Behavior<A>],
    i: &mut usize,
    cursor: &mut Box<State<A, S>>,
    f: &mut F,
) -> (Status, f64)
where
    A: Clone,
    F: FnMut(ActionArgs<A, S>) -> (Status, f64),
{
    let (status, inv_status) = if select {
        // `Select`
        (Status::Failure, Status::Success)
    } else {
        // `Sequence`
        (Status::Success, Status::Failure)
    };
    let mut remaining_dt = upd.unwrap_or(0.0);
    while *i < seq.len() {
        match cursor.event(
            match upd {
                Some(_) => Some(remaining_dt),
                _ => upd,
            },
            f,
        ) {
            (Status::Running, _) => {
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
