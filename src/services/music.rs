use poise::serenity_prelude::CreateEmbed;
use songbird::tracks::TrackHandle;
use songbird::Songbird;
use poise::serenity_prelude::GuildId;
use std::sync::Arc;

use crate::{Context, Error};

pub fn check_msg<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(why) = result {
        println!("Error sending message: {why:?}");
    }
}

pub async fn get_handler_lock(
    ctx: &Context<'_>,
) -> Result<Arc<tokio::sync::Mutex<songbird::Call>>, Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap();
    manager
        .get(guild_id)
        .ok_or_else(|| "ボイスチャンネルに参加していません".into())
}

#[derive(Clone, Debug, Default)]
pub struct TrackData {
    pub title: Option<String>,
    pub source_url: Option<String>,
    pub duration: Option<std::time::Duration>,
}

pub fn format_duration(duration: Option<std::time::Duration>) -> Option<String> {
    let total = duration?.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    Some(format!("{minutes}:{seconds:02}"))
}

pub fn build_queue_embed(tracks: &[TrackHandle]) -> CreateEmbed {
    let max_show = 15usize;
    let mut lines = Vec::new();

    for (i, handle) in tracks.iter().take(max_show).enumerate() {
        let status = if i == 0 { "再生中" } else { "待機" };
        let data = handle.data::<TrackData>();
        let title = data
            .title
            .clone()
            .or_else(|| data.source_url.clone())
            .unwrap_or_else(|| "不明".to_string());
        let duration = format_duration(data.duration);
        let line = if let Some(d) = duration {
            format!("{}. [{}] {} ({})", i + 1, status, title, d)
        } else {
            format!("{}. [{}] {}", i + 1, status, title)
        };
        lines.push(line);
    }

    if tracks.len() > max_show {
        lines.push(format!("...他 {} 件", tracks.len() - max_show));
    }

    CreateEmbed::default()
        .title(format!("再生キュー: {} 件", tracks.len()))
        .description(lines.join("\n"))
}

/// ボイスチャンネルから退出し、再生キューをすべて空にする
pub async fn clear_queue(manager: &Songbird, guild_id: GuildId) {
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
    }
}
