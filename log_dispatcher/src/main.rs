use std::io;

use telegram_bot::types::refs::ChatId;
use telegram_bot::Api;
use telegram_bot_raw::requests::send_message::CanSendMessage;

#[tokio::main]
async fn main() {
    // loading settings
    let mut settings = config::Config::new();
    settings
        .merge(config::File::with_name("LogDispatcherSettings"))
        .expect("Cannot load config from Settings.toml");

    let telegram_token = settings
        .get_str("telegram_token")
        .expect("Cannot load telegram bot token from config.");
    let chat_id = settings
        .get_int("telegram_chat_id")
        .expect("Cannot load chat id from config");

    let api = Api::new(telegram_token);
    let chat = ChatId::new(chat_id);

    println!("Running...");

    // Main loop redirecting stdin to a blocking post request
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                println!("{}", input);
                api.spawn(chat.text(input));
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
