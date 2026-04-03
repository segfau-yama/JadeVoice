use async_trait::async_trait;
use poise::serenity_prelude as serenity;
use songbird::{
    events::{CoreEvent, Event, EventContext, EventHandler as VoiceEventHandler},
    tracks::Track,
    Call, Songbird,
};
use serenity::EventHandler;
use std::sync::Arc;
use tokio::sync::Mutex;
use dashmap::DashMap;

use crate::services::clear_queue;
use voicevox_api::VoicevoxApi;

pub struct Handler {
    pub voicevox_api: VoicevoxApi,
    pub voicevox_styles: Arc<DashMap<serenity::GuildId, u16>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::Context, msg: serenity::Message) {
        if msg.author.bot {
            return;
        }

        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return,
        };

        let manager = match songbird::get(&ctx).await {
            Some(m) => m,
            None => return,
        };

        let handler_lock = match manager.get(guild_id) {
            Some(h) => h,
            None => return,
        };

        let bot_channel_id = {
            let handler = handler_lock.lock().await;
            handler.current_channel()
        };

        let bot_channel_id = match bot_channel_id {
            Some(id) => id,
            None => return,
        };

        if msg.channel_id.get() != bot_channel_id.0.get() {
            return;
        }

        let voicevox_api = self.voicevox_api.clone();
        let style_id = self
            .voicevox_styles
            .get(&guild_id)
            .map(|s| *s)
            .unwrap_or(3_u16);

        let src = match voicevox_api.tts(&msg.content, style_id as u32).await {
            Ok(wav) => wav,
            Err(_) => return,
        };

        let input: songbird::input::Input = src.into();
        let track = Track::new(input);

        let mut handler = handler_lock.lock().await;
        let _ = handler.enqueue(track).await;
    }
}

pub async fn register(
    handler_lock: &Arc<Mutex<Call>>,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    cache: Arc<serenity::Cache>,
) {
    let bot_id = cache.current_user().id;
    let evt = VoiceHandler {
        manager,
        guild_id,
        cache,
        bot_id,
    };
    let mut handler = handler_lock.lock().await;
    handler.add_global_event(CoreEvent::DriverDisconnect.into(), evt.clone());
    handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt);
}

#[derive(Clone)]
struct VoiceHandler {
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    cache: Arc<serenity::Cache>,
    bot_id: serenity::UserId,
}

#[async_trait]
impl VoiceEventHandler for VoiceHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            // ボットが切断された (強制退出・ネットワーク断など) のでキューを空にする
            EventContext::DriverDisconnect(_) => {
                clear_queue(&self.manager, self.guild_id).await;
            }

            // 同チャンネルの誰かが切断した ─ 人間がいなくなったら自動退出
            EventContext::ClientDisconnect(_) => {
                let handler_lock = match self.manager.get(self.guild_id) {
                    Some(h) => h,
                    None => return None,
                };
                let bot_channel_id = {
                    let handler = handler_lock.lock().await;
                    handler.current_channel()
                };
                let bot_channel_id = match bot_channel_id {
                    Some(id) => id,
                    None => return None,
                };
                let human_count = {
                    let guild = match self.cache.guild(self.guild_id) {
                        Some(g) => g,
                        None => return None,
                    };
                    guild
                        .voice_states
                        .values()
                        .filter(|vs| {
                            vs.channel_id
                                .map(|ch| ch.get() == bot_channel_id.0.get())
                                .unwrap_or(false)
                        })
                        .filter(|vs| vs.user_id != self.bot_id)
                        .count()
                };
                if human_count == 0 {
                    clear_queue(&self.manager, self.guild_id).await;
                    let _ = self.manager.remove(self.guild_id).await;
                }
            }

            _ => {}
        }
        None
    }
}
