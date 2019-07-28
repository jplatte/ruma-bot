use crate::{Bot, State};

// Currently, GetParam could also be implemented direclty on &'a Bot, but in the future,
// event-specific data will be added to this struct.
pub struct HandlerParamMatcher<'a> {
    pub bot: &'a Bot,
}

pub trait GetParam<T> {
    fn get(&self) -> Option<T>;
}

impl<'a> GetParam<&'a Bot> for HandlerParamMatcher<'a> {
    fn get(&self) -> Option<&'a Bot> {
        Some(self.bot)
    }
}

// TODO: There should be a way of replacing 'static with 'a here without getting an error
impl<'a, T: Clone + Send + Sync + 'static> GetParam<State<T>> for HandlerParamMatcher<'a> {
    fn get(&self) -> Option<State<T>> {
        self.bot.state.get().cloned()
    }
}
