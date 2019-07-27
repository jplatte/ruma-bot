#![feature(async_await)]

use failure::Fallible;
use ruma_bot::{command_handler, BotBuilder};

#[command_handler(command = "help")]
async fn help() -> Fallible<()> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let bot = BotBuilder::new().register(help).build();
}
