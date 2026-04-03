use poise::serenity_prelude as serenity;
use reqwest::Client;
use songbird::SerenityInit;
use dashmap::DashMap;
use dotenv::dotenv;

use voicevox_api::VoicevoxApi;

mod commands;
mod services;
mod events;
use commands::music;

pub struct Data {
    pub voicevox_api: VoicevoxApi,
    pub guild_styles: DashMap<GuildId, u16>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN");
    let dictionary = std::env::var("VOICEVOX_DICTIONARY")
        .expect("Missing VOICEVOX_DICTIONARY");
    let runtime = std::env::var("VOICEVOX_ONNXRUNTIME")
        .expect("Missing VOICEVOX_ONNXRUNTIME");

    let mut api = VoicevoxApi::new(dictionary, runtime)

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                voice(),
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
                    voicevox_api: api,
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
