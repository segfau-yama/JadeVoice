use crate::{Error, Context};
use crate::services::{check_msg};

#[poise::command(
    slash_command,
    subcommands("join", "leave", "model"),
    guild_only
)]
pub async fn voice(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, guild_only, rename = "join")]
/// ボイスチャンネルに参加する。再生中の音声がある場合は続行する。
pub async fn join(
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
/// ボイスチャンネルから退出する。再生中の音声がある場合は停止する。
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


#[poise::command(slash_command, owners_only)]
/// Voicevoxモデルを変更する
pub async fn model(
    ctx: Context<'_>,
    style_id: u16,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    ctx.data().voicevox_styles.insert(guild_id, style_id);
    check_msg(ctx.say(format!("スタイルIDを {} に変更しました", style_id)).await);
    Ok(())
}
