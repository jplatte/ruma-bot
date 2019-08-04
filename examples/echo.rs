#![feature(async_await)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use failure::Fallible;
use ruma_bot::{command_handler, Bot, BotBuilder, MsgContent, State};

type AppState = Arc<Mutex<HashMap<String, String>>>;

#[command_handler]
async fn help(bot: Bot, msg_content: MsgContent) -> Fallible<()> {
    println!("help called!");

    Ok(())
}

//#[command_handler(commands = ["x", "y", "z"])]
//async fn test(state: State<AppState>) -> Fallible<()> {
//    Ok(())
//}

//#[command_handler(command = "fetch {id}")]
//#[command_handler(command = regex("a(B|CD) (\w+)"))]

//#[reaction_handler]

#[tokio::main]
async fn main() {
    let bot = BotBuilder::new()
        //.state(Arc::new(Mutex::new(HashMap::<String, String>::new())))
        .register(help)
        .build()
        .unwrap();

    if let Err(e) = bot.run().await {
        eprintln!("{}", e);
    }
}
