#![feature(async_await)]

use failure::Fallible;
use ruma_bot::{command_handler, BotBuilder};

#[command_handler]
async fn help() -> Fallible<()> {
    Ok(())
}

#[command_handler(commands = ["x", "y", "z"])]
async fn test() -> Fallible<()> {
    Ok(())
}

//#[command_handler(command = "fetch {id}")]
//#[command_handler(command = regex("a(B|CD) (\w+)"))]

#[tokio::main]
async fn main() {
    let bot = BotBuilder::new().register(help).build();
}
