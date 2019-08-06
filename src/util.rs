// This module is not public, and all exports from it are #[doc(hidden)], so the docs wouldn't be
// visible anyway.
#![allow(missing_docs)]

use crate::{Bot, MsgContent, State};

#[derive(Debug)]
pub struct HandlerParamMatcher<'a> {
    pub bot: &'a Bot,
    pub msg_content: &'a str,
}

pub trait GetParam<T> {
    fn get(&self) -> Option<T>;
}

impl GetParam<Bot> for HandlerParamMatcher<'_> {
    fn get(&self) -> Option<Bot> {
        Some(self.bot.clone())
    }
}

// TODO: There should be a way of replacing 'static with 'a here without getting an error
impl<T: Clone + Send + Sync + 'static> GetParam<State<T>> for HandlerParamMatcher<'_> {
    fn get(&self) -> Option<State<T>> {
        self.bot.state.get().cloned()
    }
}

impl GetParam<MsgContent> for HandlerParamMatcher<'_> {
    fn get(&self) -> Option<MsgContent> {
        Some(MsgContent(self.msg_content.to_owned()))
    }
}
