use crate::status::Status::*;
use crate::tracer::{first_child_id, next_sibling_id, NodeMeta, Tracer};
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

pub struct ReactiveSequenceArgs<'a, A, E, F, B, T> {
    pub select: bool,
    pub upd: Option<Float>,
    pub seq: &'a [Behavior<A>],
    pub cursor: &'a mut Box<State<A>>,
    pub e: &'a E,
    pub blackboard: &'a mut B,
    pub f: &'a mut F,
    pub parent_id: usize,
    pub metas: &'a [NodeMeta],
    pub tracer: &'a mut T,
}

/// Shared driver for `ReactiveSequence` and `ReactiveSelect`.
///
/// Walks `seq` from index 0 every call (no resume across calls). Before
/// ticking each child, `*cursor` is overwritten with `State::new(child.clone())`
/// so the previous tick's running state is discarded.
///
/// `select` swaps short-circuit polarity:
/// - `false` → ReactiveSequence: `Failure` short-circuits; all-`Success` → `Success`.
/// - `true`  → ReactiveSelect:   `Success` short-circuits; all-`Failure` → `Failure`.
///
/// The cursor `Box` is provided by the caller and reused — this function
/// never allocates a new `Box`. Per-child allocations come solely from
/// `State::new(child.clone())` and are zero when the child is a leaf with a
/// `Copy` action type.
#[inline]
pub fn reactive_sequence<A, E, F, B, T>(args: ReactiveSequenceArgs<A, E, F, B, T>) -> (Status, Float)
where
    A: Clone,
    E: UpdateEvent,
    F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    T: Tracer,
{
    let ReactiveSequenceArgs {
        select,
        upd,
        seq,
        cursor,
        e,
        blackboard,
        f,
        parent_id,
        metas,
        tracer,
    } = args;

    let initial_dt = upd.unwrap_or(0.0);

    // Empty children: vacuous outcome.
    if seq.is_empty() {
        return if select {
            (Status::Failure, initial_dt)
        } else {
            (Status::Success, initial_dt)
        };
    }

    let (terminal_status, short_circuit_status) = if select {
        (Status::Failure, Status::Success)
    } else {
        (Status::Success, Status::Failure)
    };

    let mut child_id = first_child_id::<T>(parent_id);
    let mut remaining_dt = initial_dt;
    let mut remaining_e;

    for idx in 0..seq.len() {
        // In-place cursor reset. Drops the previous child State (may walk a
        // subtree) and constructs the new one through the existing Box —
        // the Box itself is NOT re-allocated.
        **cursor = State::new(seq[idx].clone());

        let ev = match upd {
            Some(_) => {
                remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                &remaining_e
            }
            None => e,
        };

        match cursor.tick(child_id, metas, ev, blackboard, f, tracer) {
            (Running, _) => return RUNNING,
            (s, dt) if s == short_circuit_status => return (s, dt),
            (s, dt) if s == terminal_status => {
                if upd.is_some() {
                    remaining_dt = dt;
                }
            }
            _ => unreachable!(),
        }

        if T::IS_RECORDING {
            child_id = next_sibling_id::<T>(metas, child_id);
        }
    }

    (terminal_status, remaining_dt)
}
