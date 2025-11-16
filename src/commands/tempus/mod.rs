use crate::Context;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;

pub mod link;
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
