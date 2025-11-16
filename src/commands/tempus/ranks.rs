use ::serenity::all::CreateEmbed;
use poise::{CreateReply, command, serenity_prelude as serenity};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use serenity::Mentionable;

use crate::{
    Context,
    commands::tempus::{
        get_tempus_id,
        link::{TempusPlayerInfo, TempusRankInfo},
    },
};

#[derive(Debug, Serialize, Deserialize)]
struct TempusRankData {
    player_info: TempusPlayerInfo,
    rank_info: TempusRankInfo,
    class_rank_info: ClassRankInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClassRankInfo {
    #[serde(rename = "3")]
    soldier: ClassSpecificRankInfo,
    #[serde(rename = "4")]
    demoman: ClassSpecificRankInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClassSpecificRankInfo {
    points: f64,
    rank: i64,
    total_ranked: i64,
    title: Option<String>,
}

#[command(prefix_command, aliases("srank", "drank"))]
pub async fn rank(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    let discord_id = ctx.author().id.get() as i64;

    let tempus_id = match get_tempus_id(discord_id).await {
        Ok(id) => id,
        Err(e) => {
            ctx.reply(format!("{e}")).await?;
            return Ok(());
        }
    };

    let res = reqwest::get(format!(
        "https://tempus2.xyz/api/v0/players/id/{tempus_id}/rank"
    ))
    .await?;

    if res.status() == StatusCode::TOO_MANY_REQUESTS {
        ctx.send(poise::CreateReply::default().content(format!(
            "{} ratelimited!!!! {}",
            ctx.author_member().await.unwrap().mention(),
            serenity::UserId::new(253290704384557057).mention()
        )))
        .await?;
        ctx.framework().shard_manager().shutdown_all().await;
        return Ok(());
    }

    let res: TempusRankData = serde_json::from_str(&res.text().await?)?;

    let embed = CreateEmbed::new()
        .title(format!("{}'s ranks!", res.player_info.name))
        .field(
            "Overall",
            format!(
                "{} is ranked {}/{}, with {} points!",
                res.player_info.name,
                res.rank_info.rank,
                res.rank_info.total_ranked,
                res.rank_info.points
            ),
            true,
        )
        .field(
            "Soldier",
            format!(
                "{} is ranked {}/{}, title: {}",
                res.player_info.name,
                res.class_rank_info.soldier.rank,
                res.class_rank_info.soldier.total_ranked,
                res.class_rank_info
                    .soldier
                    .title
                    .unwrap_or("unranked".to_string())
            ),
            false,
        )
        .field(
            "Demoman",
            format!(
                "{} is ranked {}/{}, title: {}",
                res.player_info.name,
                res.class_rank_info.demoman.rank,
                res.class_rank_info.demoman.total_ranked,
                res.class_rank_info
                    .demoman
                    .title
                    .unwrap_or("unranked".to_string())
            ),
            false,
        );

    ctx.send(CreateReply::default().embed(embed).reply(true))
        .await?;

    Ok(())
}
