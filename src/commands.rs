use crate::{Error, Context};
use crate::services::{TrackData, format_duration, build_queue_embed, check_msg, get_handler_lock};
use rand::seq::SliceRandom;
use songbird::input::YoutubeDl;
use songbird::tracks::Track;
use std::sync::Arc;
use std::io::Cursor;

#[poise::command(
    slash_command,
    subcommands("read", "_join", "leave", "model"),
    guild_only
)]
pub async fn voice(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "join")]
/// ボイスチャンネルに参加する。再生中の曲がある場合は続行する。
pub async fn _join(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let channel_id = ctx
        .guild()
        .unwrap()
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or("先にボイスチャンネルに参加してください")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    let handler_lock = manager.join(guild_id, channel_id).await?;
    crate::events::register(
        &handler_lock,
        manager,
        guild_id,
        ctx.serenity_context().cache.clone(),
    ).await;
    check_msg(ctx.say("ボイスチャンネルに参加しました").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// ボイスチャンネルから退出する。再生中の曲がある場合は停止する。
pub async fn leave(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    if manager.get(guild_id).is_none() {
        check_msg(ctx.say("ボイスチャンネルに参加していません").await);
        return Ok(());
    }

    manager.remove(guild_id).await?;
    check_msg(ctx.say("ボイスチャンネルから退出しました").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// 文章を読み上げる
pub async fn read(
    ctx: Context<'_>,
    sentence: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let voicevox_api = ctx.data().voicevox_api.clone();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap()
        .clone();

    let handler_lock = if let Some(existing) = manager.get(guild_id) {
        existing
    } else {
        let channel_id = ctx
            .guild()
            .unwrap()
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
            .ok_or("先にボイスチャンネルに参加してください")?;
        let joined = manager.join(guild_id, channel_id).await?;
        crate::events::register(
            &joined,
            manager.clone(),
            guild_id,
            ctx.serenity_context().cache.clone(),
        ).await;
        joined
    };

    let mut handler = handler_lock.lock().await;

    let src = voicevox_api.tts(sentence, 3).unwrap();
    let mut input: Input = src.into();
    let track = Track::new(input);
    let _ = handler.enqueue(track).await;
    Ok(())
}


#[poise::command(slash_command, owner_only)]
/// Voicevoxモデルを変更する
pub async fn model(
    ctx: Context<'_>,
    style_id: u16,
) -> Result<(), Error> {
    let tracks = queue.current_queue();
    let embed = build_queue_embed(&tracks);
    check_msg(ctx.send(poise::CreateReply::default().embed(embed)).await);
    Ok(())
}
