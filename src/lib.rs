//! `ruma_api_bot` is a crate that aims to simplify creating bots for [matrix][] servers.
//!
//! It currently requires nightly, but should work on stable once the `async_await` feature is
//! stabilized.
//!
//! [matrix]: https://matrix.org/

#![feature(async_await)]
#![warn(missing_debug_implementations, missing_docs)]

use std::{collections::HashMap, sync::Arc};

pub use ruma_bot_macro::command_handler;

/// An `async fn` annotated with `#[ruma_bot::command_handler]
pub trait CommandHandler {
    /// The command(s) this function handles
    fn commands(&self) -> &'static [&'static str];
}

#[derive(Clone)]
pub struct BotBuilder {
    handlers: HashMap<&'static str, &'static dyn CommandHandler>,
}

impl BotBuilder {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register(mut self, handler: &'static dyn CommandHandler) -> Self {
        for command in handler.commands() {
            let _old_value = self.handlers.insert(command, handler);
            // TODO: Log a warning if _old_value is Some
        }

        self
    }

    pub fn build(self) -> Bot {
        Bot {
            handlers: Arc::new(self.handlers),
        }
    }
}

pub struct Bot {
    handlers: Arc<HashMap<&'static str, &'static dyn CommandHandler>>,
}

impl Bot {
    pub async fn run(&self) -> Result<(), ()> {
        Ok(())
    }
}
