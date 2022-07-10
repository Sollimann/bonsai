#![allow(dead_code, unused_variables)]
/// Used to identify events arguments provided by traits.
///
/// Use format `<api>/<event>` to avoid naming collision.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct EventId(pub &'static str);

/// Update arguments, such as delta time in seconds.
/// To move the behavior tree forward in time it must be ticked on each iteration of the
/// game/application loop.
///
/// dt: states how much forward in time we should move the behavior tree
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateArgs {
    /// Delta time in seconds.
    pub dt: f64,
}

/// Models loop events.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum Loop {
    /// Update the state of the application.
    Update(UpdateArgs),
}

impl From<UpdateArgs> for Event {
    fn from(args: UpdateArgs) -> Self {
        Event::Loop(Loop::Update(args))
    }
}

/// Models all events.
#[derive(Clone)]
pub enum Event {
    /// Input events.
    ///
    /// Events that commonly used by event loops.
    Loop(Loop),
}

/// When the application state should be updated.
pub trait UpdateEvent: Sized {
    /// Creates an update event.
    fn from_update_args(args: &UpdateArgs, old_event: &Self) -> Option<Self>;
    /// Creates an update event with delta time.
    fn from_dt(dt: f64, old_event: &Self) -> Option<Self> {
        UpdateEvent::from_update_args(&UpdateArgs { dt }, old_event)
    }
    /// Calls closure if this is an update event.
    fn update<U, F>(&self, f: F) -> Option<U>
    where
        F: FnMut(&UpdateArgs) -> U;
    /// Returns update arguments.
    fn update_args(&self) -> Option<UpdateArgs> {
        self.update(|args| *args)
    }
}

impl UpdateEvent for Event {
    fn from_update_args(args: &UpdateArgs, _old_event: &Self) -> Option<Self> {
        Some(Event::Loop(Loop::Update(*args)))
    }

    fn update<U, F>(&self, mut f: F) -> Option<U>
    where
        F: FnMut(&UpdateArgs) -> U,
    {
        match *self {
            Event::Loop(Loop::Update(ref args)) => Some(f(args)),
        }
    }
}

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
    now: Instant,
}

impl Timer {
    /// Initialize monotonic clock
    pub fn init_time() -> Timer {
        let init = Instant::now();
        Timer { start: init, now: init }
    }

    /// Compute duration since timer started
    pub fn duration_since_start(&self) -> f64 {
        let new_now: Instant = Instant::now();
        let duration = new_now.duration_since(self.start);
        duration.as_secs_f64()
    }

    /// Compute time difference last invocation of `get_dt()` function
    pub fn get_dt(&mut self) -> f64 {
        let new_now: Instant = Instant::now();
        let duration = new_now.duration_since(self.now);
        self.now = new_now;
        duration.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_update_args() {
        use Event;
        use UpdateArgs;

        let e: Event = UpdateArgs { dt: 0.0 }.into();
        let _: Option<Event> = UpdateEvent::from_update_args(&UpdateArgs { dt: 1.0 }, &e);
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::init_time();
        sleep(Duration::new(0, 0.1e+9 as u32));
        let duration = timer.duration_since_start();
        let dt = timer.get_dt();

        assert!(duration < 1.0);
        assert!(dt < 0.2);
        assert!(dt >= 0.1);

        sleep(Duration::new(0, 0.3e+9 as u32));
        let duration = timer.duration_since_start();
        let dt = timer.get_dt();

        assert!(duration < 1.0);
        assert!(duration > 0.3);
        assert!(dt > 0.2);
        assert!(dt < 0.4);
    }
}
