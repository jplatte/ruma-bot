#![feature(async_await)]

use ruma_bot::{command_handler, BotBuilder};

#[command_handler(command = "help")]
async fn help() {}

#[tokio::main]
async fn main() {
    let bot = BotBuilder::new().register(&help).build();
}
