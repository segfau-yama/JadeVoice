use songbird::Songbird;
use poise::serenity_prelude::GuildId;

pub fn check_msg<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(why) = result {
        println!("Error sending message: {why:?}");
    }
}


/// ボイスチャンネルから退出し、再生キューをすべて空にする
pub async fn clear_queue(manager: &Songbird, guild_id: GuildId) {
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
    }
}
