//! `ruma_api_bot` is a crate that aims to simplify creating bots for [matrix][] servers.
//!
//! It currently requires nightly, but should work on stable once the `async_await` feature is
//! stabilized.
//!
//! [matrix]: https://matrix.org/

#![feature(async_await)]
#![warn(missing_debug_implementations, missing_docs)]

use std::{collections::HashMap, sync::Arc};

use anymap::any::Any;
type AnyMap = anymap::Map<dyn Any + Send + Sync>;

pub use ruma_bot_macro::command_handler;

mod state;

/// A function (usually `async`) annotated with `#[ruma_bot::command_handler]
pub trait CommandHandler {
    /// The command(s) this function handles
    fn commands(&self) -> &'static [&'static str];

    fn get_fn(&self) -> Box<dyn CommandHandlerFn>;
}

pub trait CommandHandlerFn: Send + Sync {
    //fn
}

pub struct BotBuilder {
    handlers: HashMap<&'static str, Box<dyn CommandHandlerFn>>,
}

impl BotBuilder {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register(mut self, handler: impl CommandHandler) -> Self {
        for command in handler.commands() {
            let _old_value = self.handlers.insert(command, handler.get_fn());
            // TODO: Log a warning if _old_value is Some
        }

        self
    }

    pub fn build(self) -> Bot {
        Bot {
            handlers: Arc::new(self.handlers),
            state: AnyMap::new(),
        }
    }
}

pub struct Bot {
    handlers: Arc<HashMap<&'static str, Box<dyn CommandHandlerFn>>>,
    state: AnyMap,
}

impl Bot {
    pub async fn run(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[allow(dead_code)]
mod compile_tests {
    use super::Bot;

    fn send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Send>() {}
        assert_send::<Bot>();
        assert_sync::<Bot>();
    }
}
