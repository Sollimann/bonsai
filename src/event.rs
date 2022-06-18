/// Used to identify events arguments provided by traits.
///
/// Use format `<api>/<event>` to avoid naming collision.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct EventId(pub &'static str);

/// Update arguments, such as delta time in seconds.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_args() {
        use Event;
        use UpdateArgs;

        let e: Event = UpdateArgs { dt: 0.0 }.into();
        let _: Option<Event> = UpdateEvent::from_update_args(&UpdateArgs { dt: 1.0 }, &e);
    }
}
