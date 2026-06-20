//! Proves that ticking a memoryless `Sequence` (`memory = false`) with leaf
//! `Copy` children does zero heap allocations after a warmup tick.
//!
//! # Why counting is per-thread
//!
//! `#[global_allocator]` is process-wide and Cargo runs tests in parallel.
//! A shared counter would catch every other test's allocations too, so we
//! gate the count behind a thread-local flag that's only set inside this test.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use bonsai_bt::{Action, ActionArgs, Event, Sequence, Status, UpdateArgs, BT};

/// Fast process-wide check: when false, skip the per-thread lookup entirely.
/// One relaxed atomic load per allocation.
static ANY_THREAD_MEASURING: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// True only on the test thread while measuring.
    /// `const` init means the first access on a thread doesn't allocate.
    static THIS_THREAD_MEASURING: Cell<bool> = const { Cell::new(false) };
    /// Per-thread allocation tally.
    static THIS_THREAD_COUNT: Cell<u64> = const { Cell::new(0) };
}

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ANY_THREAD_MEASURING.load(Ordering::Relaxed) {
            THIS_THREAD_MEASURING.with(|active| {
                if active.get() {
                    THIS_THREAD_COUNT.with(|c| c.set(c.get() + 1));
                }
            });
        }
        // SAFETY: passing the caller's layout straight to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: ptr came from System.alloc above, with the same layout.
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator;

#[test]
fn memoryless_sequence_steady_state_is_zero_alloc() {
    // Two leaf children with `Copy` actions (i32). The second always returns
    // Running, so the composite never finishes and we never call `reset_bt` —
    // which itself allocates a fresh State tree.
    //
    // Action codes: 0 = Success, 1 = Running.
    let tree = Sequence(vec![Action(0_i32), Action(1_i32)]).memory(false);
    let mut bt = BT::new(tree, ());

    let e: Event = UpdateArgs { dt: 0.0 }.into();
    let mut step = |args: ActionArgs<Event, i32>, _bb: &mut ()| match *args.action {
        0 => (Status::Success, args.dt),
        1 => (Status::Running, 0.0),
        _ => unreachable!(),
    };

    // Warmup tick — the first run may allocate (e.g. telemetry setup).
    let _ = bt.tick(&e, &mut step);

    // Start measuring. Flip the thread-local flag before the global one so
    // any allocation between these two lines is still counted.
    THIS_THREAD_COUNT.with(|c| c.set(0));
    THIS_THREAD_MEASURING.with(|c| c.set(true));
    ANY_THREAD_MEASURING.store(true, Ordering::Relaxed);

    for _ in 0..100 {
        let _ = bt.tick(&e, &mut step);
    }

    // Stop measuring — unwind in reverse order so nothing slips through.
    ANY_THREAD_MEASURING.store(false, Ordering::Relaxed);
    THIS_THREAD_MEASURING.with(|c| c.set(false));

    let allocs = THIS_THREAD_COUNT.with(|c| c.get());
    assert_eq!(
        allocs, 0,
        "memoryless composite tick path must not allocate with leaf Copy children; \
         observed {allocs} allocations across 100 ticks"
    );
}
