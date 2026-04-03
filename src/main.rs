use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use serenity::model::id::GuildId;
use dashmap::DashMap;
use dotenv::dotenv;
use std::sync::Arc;

use voicevox_api::VoicevoxApi;

mod commands;
mod services;
mod events;
use commands::voice;
use crate::events::Handler;

pub struct Data {
    pub voicevox_styles: Arc<DashMap<GuildId, u16>>,
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

    let api = VoicevoxApi::new(&dictionary, &runtime)
        .expect("Failed to initialize VoicevoxApi");
    
    let styles: Arc<DashMap<GuildId, u16>> = Arc::new(DashMap::new());
    let styles_clone = styles.clone();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                voice(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let styles_for_setup = styles_clone.clone();
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands
                ).await?;
                Ok(Data {
                    voicevox_styles: styles_for_setup,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .event_handler(Handler {
            voicevox_api: api.clone(),
            voicevox_styles: styles.clone(),
        })
        .await?;

    client.start().await?;

    Ok(())
}
