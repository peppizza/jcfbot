use std::{env, fmt::Display, time::Duration};

use hhmmss::Hhmmss;
use poise::command;
use serde_derive::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};

use crate::{Context, commands::tempus::link::TempusPlayerInfo};

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn stime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, map, Classes::Soldier).await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dtime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, map, Classes::Demoman).await
}

#[derive(Deserialize, Serialize, Debug)]
struct TempusResultData {
    player_info: TempusPlayerInfo,
    rank: i64,
    duration: f64,
}

#[derive(Deserialize, Serialize, Debug)]
struct TempusCompletionInfo {
    soldier: i64,
    demoman: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct TempusCompletionData {
    completion_info: TempusCompletionInfo,
    result: Option<TempusResultData>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Classes {
    Soldier = 3,
    Demoman = 4,
}

impl Display for Classes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

async fn time(ctx: Context<'_>, map: String, class: Classes) -> Result<(), anyhow::Error> {
    let discord_id = ctx.author().id.get() as i64;

    let tempus_id = {
        let mut conn = SqliteConnection::connect(&env::var("DATABASE_URL").unwrap()).await?;

        let res = sqlx::query!(
            "SELECT tempus_id FROM ids WHERE discord_id = ?1",
            discord_id
        )
        .fetch_optional(&mut conn)
        .await?;

        if res.is_none() {
            ctx.reply("Tempus ID not linked!").await?;
            return Ok(());
        }

        res.unwrap().tempus_id
    };

    let res = reqwest::get(format!("https://tempus2.xyz/api/v0/maps/name/{map}/zones/typeindex/map/1/records/player/{tempus_id}/{}", class as usize)).await?.text().await?;

    let completion_data: TempusCompletionData = serde_json::from_str(&res)?;
    let completions = if class == Classes::Soldier {
        completion_data.completion_info.soldier
    } else {
        completion_data.completion_info.demoman
    };

    if let Some(completion_data) = completion_data.result {
        let time = Duration::from_secs_f64(completion_data.duration);

        ctx.reply(format!(
            "{} is ranked {}/{} on {}, {}",
            completion_data.player_info.name,
            completion_data.rank,
            completions,
            map,
            time.hhmmssxxx()
        ))
        .await?;
    } else {
        ctx.reply(format!("No {class} completion on {map} :("))
            .await?;
    }

    Ok(())
}
