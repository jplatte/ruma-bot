//! `ruma_api_bot` is a crate that aims to simplify creating bots for [matrix][] servers.
//!
//! It currently requires nightly, but should work on stable once the `async_await` feature is
//! stabilized.
//!
//! [matrix]: https://matrix.org/

#![warn(missing_debug_implementations, missing_docs)]

use std::{collections::HashMap, pin::Pin, sync::Arc};

use anymap::any::CloneAny;
use failure::{err_msg, Fallible};
use futures::{Future, TryStreamExt};
use ruma_client::{api, HttpsClient as MatrixClient};
use url::Url;

// TODO: Get rid of compiler warnings, either by replacing CloneAny, or by fixing the issue upstream
type AnyMap = anymap::Map<dyn CloneAny + Send + Sync>;
type HandlerFnMap = HashMap<&'static str, Box<dyn CommandHandler>>;

pub use ruma_bot_macros::command_handler;

mod debug;
mod util;
mod wrap;

#[doc(hidden)]
pub use util::{GetParam, HandlerParamMatcher};
pub use wrap::{MsgContent, State};

fn env_var(name: &'static str) -> Option<String> {
    std::env::var(name)
        .map_err(|e| {
            if let std::env::VarError::NotUnicode(_) = e {
                panic!("ruma_bot: {} contains non-unicode bytes", name);
            }
        })
        .ok()
}

/// A function (usually `async`) annotated with `#[ruma_bot::command_handler]
pub trait CommandHandler: Send + Sync {
    /// The command(s) this function handles
    fn commands() -> &'static [&'static str]
    where
        Self: Sized;

    /// Used to call the command handler with the necessary application state and command context
    fn handle(&self, _: &Bot, msg_content: &str) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[derive(Clone, Debug)]
struct ConnectionDetails {
    //homeserver_url: Url,
    username: String,
    password: String,
}

/// A builder for the `Bot` type
pub struct BotBuilder {
    handlers: HandlerFnMap,
    state: AnyMap,
    homeserver_url: Option<Url>,
    username: Option<String>,
    password: Option<String>,
}

impl BotBuilder {
    /// Crate a new `BotBuilder`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            state: AnyMap::new(),
            homeserver_url: None,
            username: None,
            password: None,
        }
    }

    /// Register a command handler
    pub fn register<T>(mut self, handler: T) -> Self
    where
        T: CommandHandler + Copy + 'static,
    {
        for command in T::commands() {
            let old_value = self.handlers.insert(command, Box::new(handler));
            assert!(
                old_value.is_none(),
                "ruma_bot: Tried to register a command handler for '{}' when one was already registered",
                command,
            );
        }

        self
    }

    /// Register some state to be used across calls to event handlers
    pub fn state<T>(mut self, initial_value: T) -> Self
    where
        T: CloneAny + Send + Sync,
    {
        self.state.insert(initial_value);
        self
    }

    /// Set the homeserver url (must be an `https` url currently)
    ///
    /// If the homeserver url is not set with this method, `build` will fall back to the environment
    /// variable `RUMA_BOT_HOMESERVER_URL`.
    pub fn homeserver_url(mut self, homeserver_url: Url) -> Self {
        self.homeserver_url = Some(homeserver_url);
        self
    }

    /// Set the username
    ///
    /// If the username is not set with this method, `build` will fall back to the environment
    /// variable `RUMA_BOT_USERNAME`.
    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// Set the password
    ///
    /// If the password is not set with this method, ,`build` will fall back to the environment
    /// variable `RUMA_BOT_PASSWORD`.
    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Build the bot
    pub fn build(self) -> Fallible<Bot> {
        Ok(Bot {
            client: MatrixClient::https(
                self.homeserver_url
                    .or_else(|| {
                        env_var("RUMA_BOT_HOMESERVER_URL")
                            .map(|s| s.parse().expect("valid url for RUMA_BOT_HOMESERVER_URL"))
                    })
                    .ok_or_else(|| err_msg("ruma_bot: homeserver_url not configured"))?,
                None,
            )?,
            handlers: Arc::new(self.handlers),
            state: self.state,
            connection_details: Some(ConnectionDetails {
                username: self
                    .username
                    .or_else(|| env_var("RUMA_BOT_USERNAME"))
                    .ok_or_else(|| err_msg("ruma_bot: username not configured"))?,
                password: self
                    .password
                    .or_else(|| env_var("RUMA_BOT_PASSWORD"))
                    .ok_or_else(|| err_msg("ruma_bot: password not configured"))?,
            }),
        })
    }
}

/// A bot, usually containing one or more command handlers and optionally a reaction handler,
/// which can also manage arbitrary application data (one instance per type).
#[derive(Clone)]
pub struct Bot {
    client: MatrixClient,
    handlers: Arc<HandlerFnMap>,
    state: AnyMap,
    connection_details: Option<ConnectionDetails>,
}

impl Bot {
    /// Start the Bot's main loop
    pub async fn run(mut self) -> Result<(), ruma_client::Error> {
        use api::r0::{
            filter::{Filter, FilterDefinition, RoomEventFilter, RoomFilter},
            sync::sync_events::Filter as SyncFilter,
        };
        use ruma_client::events::{
            collections::all::RoomEvent,
            room::message::{MessageEvent, MessageEventContent, TextMessageEventContent},
        };

        // A Bot is always instantiated with connection_details = Some(...) and this method (that
        // can only be called once) is the only one method that uses connection_details. Thus, this
        // .unwrap() will never fail.
        let cd = self.connection_details.take().unwrap();

        self.client.log_in(cd.username, cd.password, None).await?;

        let mut sync0 = Box::pin(self.client.sync(
            Some(SyncFilter::FilterDefinition(FilterDefinition::ignore_all())),
            None,
            false,
        ));

        let sync_start = sync0.try_next().await?.expect("sync response").next_batch;
        let mut sync_stream = Box::pin(self.client.sync(
            Some(SyncFilter::FilterDefinition(FilterDefinition {
                account_data: Some(Filter::ignore_all()),
                room: Some(RoomFilter {
                    account_data: Some(RoomEventFilter::ignore_all()),
                    state: Some(RoomEventFilter::ignore_all()),
                    ..Default::default()
                }),
                presence: Some(Filter::ignore_all()),
                ..Default::default()
            })),
            Some(sync_start),
            true,
        ));

        while let Some(res) = sync_stream.try_next().await? {
            for (_room_id, room) in res.rooms.join {
                for event in room
                    .timeline
                    .events
                    .into_iter()
                    // ignore invalid events
                    .flat_map(|ev_res| ev_res.into_result())
                {
                    // Filter out the text messages
                    if let RoomEvent::RoomMessage(MessageEvent {
                        content: MessageEventContent::Text(TextMessageEventContent { body, .. }),
                        sender: _sender,
                        ..
                    }) = event
                    {
                        if body.starts_with('!') {
                            if let Some(idx) = body.find(char::is_whitespace) {
                                let command = &body[1..idx];

                                if let Some(handler) = self.handlers.get(command) {
                                    tokio::spawn(handler.handle(&self, &body[idx + 1..]));
                                }
                            }
                        }
                    }
                }
            }

            for (_room_id, _invited_room) in res.rooms.invite {
                // TODO
            }
        }

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
