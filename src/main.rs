use poise::serenity_prelude as serenity;
use reqwest::Client;

use songbird::SerenityInit;

mod commands;
mod services;
mod events;
use commands::music::music;

use dotenv::dotenv;

pub struct Data {
    pub http_client: Client,
    pub ytdlp_cookies: String,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN");
    let cookies = std::env::var("YTDLP_COOKIES")
        .expect("YTDLP_COOKIES environment variable not set");

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                music(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands
                ).await?;
                Ok(Data {
                    http_client: Client::new(),
                    ytdlp_cookies: cookies,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await?;

    client.start().await?;

    Ok(())
}
