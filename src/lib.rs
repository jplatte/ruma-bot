//! `ruma_api_bot` is a crate that aims to simplify creating bots for [matrix][] servers.
//!
//! It currently requires nightly, but should work on stable once the `async_await` feature is
//! stabilized.
//!
//! [matrix]: https://matrix.org/

#![feature(async_await)]
#![warn(missing_debug_implementations, missing_docs)]

use std::{collections::HashMap, sync::Arc};

use anymap::any::CloneAny;
use failure::{err_msg, Fallible};
use futures::{Future, TryStreamExt};
use js_int::UInt;
use ruma_client::{api, HttpsClient as MatrixClient};
use url::Url;

// TODO: Get rid of compiler warnings, either by replacing CloneAny, or by fixing the issue upstream
type AnyMap = anymap::Map<dyn CloneAny + Send + Sync>;
type HandlerFnMap = HashMap<&'static str, Box<dyn CommandHandler>>;

pub use ruma_bot_macro::command_handler;

mod state;
mod util;

pub use state::State;
#[doc(hidden)]
pub use util::{GetParam, HandlerParamMatcher};

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
    // TODO: Additional parameter(s) for command data
    fn handle(&self, _: &Bot) -> Box<dyn Future<Output = Result<(), failure::Error>>>;
}

struct ConnectionDetails {
    //homeserver_url: Url,
    username: String,
    password: String,
}

pub struct BotBuilder {
    handlers: HandlerFnMap,
    homeserver_url: Option<Url>,
    username: Option<String>,
    password: Option<String>,
}

impl BotBuilder {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
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

    pub fn homeserver_url(mut self, homeserver_url: Url) -> Self {
        self.homeserver_url = Some(homeserver_url);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn build(self) -> Fallible<Bot> {
        Ok(Bot {
            client: MatrixClient::https(
                self.homeserver_url
                    .or_else(|| {
                        env_var("RUMA_BOT_HOMESERVER_URL")
                            .map(|s| s.parse().expect("valid url for RUMA_BOT_HOMESERVER_URL"))
                    })
                    .ok_or(err_msg("ruma_bot: homeserver_url not configured"))?,
                None,
            )?,
            handlers: Arc::new(self.handlers),
            state: AnyMap::new(),
            connection_details: Some(ConnectionDetails {
                username: self
                    .username
                    .or_else(|| env_var("RUMA_BOT_USERNAME"))
                    .ok_or(err_msg("ruma_bot: username not configured"))?,
                password: self
                    .password
                    .or_else(|| env_var("RUMA_BOT_PASSWORD"))
                    .ok_or(err_msg("ruma_bot: password not configured"))?,
            }),
        })
    }
}

pub struct Bot {
    client: MatrixClient,
    handlers: Arc<HandlerFnMap>,
    state: AnyMap,
    connection_details: Option<ConnectionDetails>,
}

impl Bot {
    pub async fn run(mut self) -> Result<(), ruma_client::Error> {
        // A Bot is always instantiated with connection_details = Some(...) and this method (that
        // can only be called once) is the only one method that uses connection_details. Thus, this
        // .unwrap() will never fail.
        let cd = self.connection_details.take().unwrap();

        self.client.log_in(cd.username, cd.password, None).await?;

        use api::r0::{
            filter::{Filter, FilterDefinition, RoomEventFilter, RoomFilter},
            sync::sync_events::Filter as SyncFilter,
        };

        let mut sync0 = Box::pin(self.client.sync(
            Some(SyncFilter::FilterDefinition(FilterDefinition {
                event_fields: Vec::new(),
                event_format: None,
                account_data: Some(Filter {
                    limit: Some(UInt::from(0u32)),
                    //..Filter::default()
                    senders: Vec::new(),
                    not_senders: Vec::new(),
                    types: Vec::new(),
                    not_types: Vec::new(),
                }),
                room: Some(RoomFilter {
                    account_data: Some(RoomEventFilter {
                        limit: Some(UInt::from(0u32)),
                        //..RoomEventFilter::default()
                        rooms: Vec::new(),
                        not_rooms: Vec::new(),
                        senders: Vec::new(),
                        not_senders: Vec::new(),
                        types: Vec::new(),
                        not_types: Vec::new(),
                    }),
                    timeline: Some(RoomEventFilter {
                        limit: Some(UInt::from(0u32)),
                        //..RoomEventFilter::default()
                        rooms: Vec::new(),
                        not_rooms: Vec::new(),
                        senders: Vec::new(),
                        not_senders: Vec::new(),
                        types: Vec::new(),
                        not_types: Vec::new(),
                    }),
                    ephemeral: Some(RoomEventFilter {
                        limit: Some(UInt::from(0u32)),
                        //..RoomEventFilter::default()
                        rooms: Vec::new(),
                        not_rooms: Vec::new(),
                        senders: Vec::new(),
                        not_senders: Vec::new(),
                        types: Vec::new(),
                        not_types: Vec::new(),
                    }),
                    state: Some(RoomEventFilter {
                        limit: Some(UInt::from(0u32)),
                        //..RoomEventFilter::default()
                        rooms: Vec::new(),
                        not_rooms: Vec::new(),
                        senders: Vec::new(),
                        not_senders: Vec::new(),
                        types: Vec::new(),
                        not_types: Vec::new(),
                    }),
                    //..RoomFilter::default()
                    include_leave: None,
                    rooms: Vec::new(),
                    not_rooms: Vec::new(),
                }),
                presence: Some(Filter {
                    limit: Some(UInt::from(0u32)),
                    //..Filter::default()
                    senders: Vec::new(),
                    not_senders: Vec::new(),
                    types: Vec::new(),
                    not_types: Vec::new(),
                }),
            })),
            None,
            false,
        ));

        let sync_start = sync0.try_next().await?.expect("sync response").next_batch;
        let mut sync_stream = Box::pin(self.client.sync(None, Some(sync_start), true));

        while let Some(res) = sync_stream.try_next().await? {}

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
