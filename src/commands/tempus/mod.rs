use crate::Context;

pub mod link;
pub mod times;

#[poise::command(prefix_command, owners_only)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}
