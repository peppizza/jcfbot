use crate::Context;
use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use sqlx::{Connection, SqliteConnection};
use std::env;

pub mod link;
pub mod ranks;
pub mod times;

#[poise::command(prefix_command, owners_only)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    ctx.send(poise::CreateReply::default().content(format!(
        "{} ratelimited!!!! {}",
        ctx.author_member().await.unwrap().mention(),
        serenity::UserId::new(253290704384557057).mention()
    )))
    .await?;
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

pub async fn get_tempus_id(discord_id: i64) -> Result<i64, anyhow::Error> {
    let mut conn = SqliteConnection::connect(&env::var("DATABASE_URL").unwrap()).await?;

    let res = sqlx::query!(
        "SELECT tempus_id FROM ids WHERE discord_id = ?1",
        discord_id
    )
    .fetch_optional(&mut conn)
    .await?;

    if res.is_none() {
        return Err(anyhow!("Tempus ID not linked!"));
    }

    Ok(res.unwrap().tempus_id)
}
