use std::fmt;

use crate::{Bot, BotBuilder, HandlerFnMap, MsgContent, State};

struct Placeholder;

impl fmt::Debug for Placeholder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[...]")
    }
}

struct HandlerFnMapDbg<'a>(&'a HandlerFnMap);

impl<'a> fmt::Debug for HandlerFnMapDbg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, _)| (k, Placeholder)))
            .finish()
    }
}

impl fmt::Debug for BotBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotBuilder")
            .field("handlers", &HandlerFnMapDbg(&self.handlers))
            .field("homeserver_url", &self.homeserver_url)
            .field("username", &self.username)
            .field("password", &Placeholder)
            .finish()
    }
}

impl fmt::Debug for Bot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bot")
            .field("client", &self.client)
            .field("handlers", &HandlerFnMapDbg(&self.handlers))
            .field("state", &self.state)
            .field("connection_details", &self.connection_details)
            .finish()
    }
}

impl<T> fmt::Debug for State<T>
where
    T: fmt::Debug + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Debug for MsgContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
