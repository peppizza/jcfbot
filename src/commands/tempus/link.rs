use std::{env, time::Duration};

use ::serenity::all::{
    ComponentInteractionCollector, CreateButton, CreateInteractionResponse, Mentionable, UserId,
};
use poise::{CreateReply, command, serenity_prelude as serenity};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use serenity::all::CreateEmbed;
use sqlx::{Connection, SqliteConnection};

use crate::Context;

#[derive(Serialize, Deserialize, Debug)]
pub struct TempusPlayerInfo {
    pub name: String,
    pub id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TempusRankInfo {
    points: f64,
    rank: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TempusPlayerStats {
    player_info: TempusPlayerInfo,
    rank_info: TempusRankInfo,
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn link(
    ctx: Context<'_>,
    #[description = "Tempus ID"] tempus_id: i64,
) -> Result<(), anyhow::Error> {
    let uuid = ctx.id();
    let discord_id = ctx.author().id.get() as i64;

    {
        let mut conn = SqliteConnection::connect(&env::var("DATABASE_URL").unwrap()).await?;

        let res = sqlx::query!(
            r#"SELECT tempus_id FROM ids WHERE discord_id = ?1"#,
            discord_id
        )
        .fetch_optional(&mut conn)
        .await?;

        if let Some(row) = res {
            ctx.reply(format!("Tempus ID ({}) already linked!", row.tempus_id))
                .await?;
            return Ok(());
        }
    }

    let response = reqwest::get(format!(
        "https://tempus2.xyz/api/v0/players/id/{}/stats",
        tempus_id
    ))
    .await?;

    match response.status() {
        StatusCode::TOO_MANY_REQUESTS => {
            ctx.send(CreateReply::default().content(format!(
                "{} ratelimited!!!! {}",
                ctx.author_member().await.unwrap().mention(),
                UserId::new(253290704384557057).mention()
            )))
            .await?;
            ctx.framework().shard_manager().shutdown_all().await;
            return Ok(());
        }
        StatusCode::NOT_FOUND => {
            ctx.reply("Not a valid tempus id!").await?;
            return Ok(());
        }
        _ => {}
    };

    let player_stats: TempusPlayerStats = serde_json::from_str(&response.text().await?)?;

    let reply = {
        let embed = CreateEmbed::new()
            .title(format!("Found player: {}", player_stats.player_info.name))
            .description(format!(
                "Overall rank: {}, Overall points: {}",
                player_stats.rank_info.rank, player_stats.rank_info.points
            ));

        let components = vec![serenity::CreateActionRow::Buttons(vec![
            CreateButton::new(format!("{uuid}.yes"))
                .label("dat me")
                .style(serenity::ButtonStyle::Success),
            CreateButton::new(format!("{uuid}.no"))
                .label("dat not me...")
                .style(serenity::ButtonStyle::Danger),
        ])];

        CreateReply::default().embed(embed).components(components)
    };

    ctx.send(reply).await?;

    while let Some(choice) = ComponentInteractionCollector::new(ctx)
        .channel_id(ctx.channel_id())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(10))
        .filter(move |choice| {
            choice.data.custom_id == format!("{uuid}.yes")
                || choice.data.custom_id == format!("{uuid}.no")
        })
        .await
    {
        if choice.data.custom_id == format!("{uuid}.yes") {
            let mut conn = SqliteConnection::connect(&env::var("DATABASE_URL").unwrap()).await?;

            sqlx::query!(
                r#"
                INSERT OR FAIL INTO ids (discord_id, tempus_id) VALUES (?1, ?2);
            "#,
                discord_id,
                tempus_id
            )
            .execute(&mut conn)
            .await?;

            choice
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;

            ctx.reply("Successfully linked your tempus id!").await?;
        } else {
            choice
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;

            ctx.reply("gg...").await?;
        }

        choice.message.delete(ctx).await?;
    }

    Ok(())
}
