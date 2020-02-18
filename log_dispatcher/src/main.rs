use std::io;

use reqwest::blocking::{multipart::Form, Client};

fn main() {
    // loading settings
    let mut settings = config::Config::new();
    settings
        .merge(config::File::with_name("Settings"))
        .expect("Cannot load config from Settings.toml");
    let client = Client::new();

    let url = settings
        .get_str("discord_webhook_url")
        .expect("Cannot load url from config.");

    // Sending test request
    let form = Form::new().text("content", "Webhook collector started running.");
    // TODO gracefully catch an error instead of panicking
    client.post(&url).multipart(form).send().unwrap();

    println!("Running...");

    // Main loop redirecting stdin to a blocking post request
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let form = Form::new().text("content", input);
                // TODO gracefully catch an error instead of panicking
                client.post(&url).multipart(form).send().unwrap();
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
