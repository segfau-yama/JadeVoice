use crate::{Error, Context};
use crate::services::music::{TrackData, format_duration, build_queue_embed, check_msg, get_handler_lock};
use rand::seq::SliceRandom;
use songbird::input::YoutubeDl;
use songbird::tracks::Track;
use std::sync::Arc;

#[poise::command(
    slash_command,
    subcommands("play", "skip", "_join", "leave", "shuffle", "list", "delete"),
    guild_only
)]
pub async fn music(_ctx: Context<'_>) -> Result<(), Error> {
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
/// 曲を再生キューに追加する。
pub async fn play(
    ctx: Context<'_>,
    #[description = "再生するURL (YouTube etc…)"]
    url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let http_client = ctx.data().http_client.clone();

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

    let cookies = ctx.data().ytdlp_cookies.clone();
    println!("Using ytdlp cookies: {}", cookies);
    println!("URL: {}", url);
    let extra_args = vec![
        "--cookies".to_string(), cookies,
        "--user-agent".to_string(), "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0".to_string(),
        "--remote-components".to_string(), "ejs:github".to_string(),
        "--js-runtime".to_string(), "deno".to_string(),
    ];

    let do_search = !url.starts_with("http");
    let src = if do_search {
        YoutubeDl::new_search(http_client, url)
    } else {
        YoutubeDl::new(http_client, url)
    }
    .user_args(extra_args);
    let mut input: songbird::input::Input = src.into();
    let aux = input.aux_metadata().await.ok();
    let track_data = Arc::new(TrackData {
        title: aux.as_ref().and_then(|m| m.title.clone()),
        source_url: aux.as_ref().and_then(|m| m.source_url.clone()),
        duration: aux.as_ref().and_then(|m| m.duration),
    });
    let track = Track::new_with_data(input, track_data.clone());
    let _ = handler.enqueue(track).await;

    let title = track_data
        .title
        .clone()
        .unwrap_or_else(|| "不明なタイトル".to_string());
    let mut embed = poise::serenity_prelude::CreateEmbed::default()
        .title("再生キューに追加しました")
        .field("タイトル", &title, false);
    if let Some(url) = &track_data.source_url {
        embed = embed.url(url);
        embed = embed.field("URL", url, false);
    }
    if let Some(d) = format_duration(track_data.duration) {
        embed = embed.field("再生時間", d, true);
    }
    check_msg(ctx.send(poise::CreateReply::default().embed(embed)).await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// 再生中の曲をスキップする。次の曲があれば再生を開始する。
pub async fn skip(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let handler_lock = get_handler_lock(&ctx).await?;

    handler_lock.lock().await.queue().skip()?;
    check_msg(ctx.say("スキップしました").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// 再生キューをシャッフルする。再生中の曲はそのまま続行する。
pub async fn shuffle(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let handler_lock = get_handler_lock(&ctx).await?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let len = queue.len();

    if len <= 1 {
        check_msg(ctx.say("シャッフルできる曲がありません").await);
        return Ok(());
    }

    queue.modify_queue(|vq| {
        let mut rng = rand::thread_rng();
        let slice = vq.make_contiguous();

        if slice.len() > 1 {
            slice[1..].shuffle(&mut rng);
        }
    });

    let tracks = queue.current_queue();
    let embed = build_queue_embed(&tracks);
    check_msg(ctx.send(poise::CreateReply::default().embed(embed)).await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// 再生キューの曲一覧を表示する。
pub async fn list(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let handler_lock = get_handler_lock(&ctx).await?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let tracks = queue.current_queue();

    if tracks.is_empty() {
        check_msg(ctx.say("再生キューは空です").await);
        return Ok(());
    }

    let embed = build_queue_embed(&tracks);
    check_msg(ctx.send(poise::CreateReply::default().embed(embed)).await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
/// 再生キューから指定した曲を削除する。
pub async fn delete(
    ctx: Context<'_>,
    #[description = "削除する曲番号 (list の 1 始まり)"] index: usize,
) -> Result<(), Error> {
    let handler_lock = get_handler_lock(&ctx).await?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let len = queue.len();

    if len == 0 {
        check_msg(ctx.say("再生キューは空です").await);
        return Ok(());
    }

    if index == 0 || index > len {
        check_msg(ctx.say(format!("無効な番号です。1 から {len} の範囲で指定してください")).await);
        return Ok(());
    }

    if index == 1 {
        check_msg(ctx.say("再生中の曲は削除できません。/music skip を使ってください").await);
        return Ok(());
    }

    let removed = queue.dequeue(index - 1);
    if let Some(track) = removed {
        let _ = track.stop();
        check_msg(ctx.say(format!("{} 番目の曲を削除しました", index)).await);
    } else {
        check_msg(ctx.say("削除対象を見つけられませんでした").await);
    }

    Ok(())
}
