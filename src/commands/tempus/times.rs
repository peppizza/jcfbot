use std::{env, fmt::Display, time::Duration};

use ::serenity::all::{CreateButton, CreateEmbed, CreateInteractionResponse};
use hhmmss::Hhmmss;
use poise::{CreateReply, command, serenity_prelude as serenity};
use serde_derive::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};

use crate::{Context, commands::tempus::link::TempusPlayerInfo};

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn stime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, &map, Classes::Soldier).await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dtime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, &map, Classes::Demoman).await
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

#[derive(Deserialize, Serialize, Debug)]
struct TempusSearchResult {
    maps: Vec<TempusMapSearch>,
}

#[derive(Deserialize, Serialize, Debug)]
struct TempusMapSearch {
    name: String,
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

async fn time(ctx: Context<'_>, map: &str, class: Classes) -> Result<(), anyhow::Error> {
    let uuid = ctx.id();
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

    let search = reqwest::get(format!(
        "https://tempus2.xyz/api/v0/search/playersAndMaps/{map}"
    ))
    .await?
    .text()
    .await?;

    let map_search: TempusSearchResult = serde_json::from_str(&search)?;
    let map_search = map_search.maps;
    let map_search_len = map_search.len();

    if map_search_len == 0 {
        ctx.reply(format!("No maps found with name {map}!")).await?;
        return Ok(());
    }

    let map_name = {
        if map_search_len > 1 {
            let map_list_reply = {
                let components = vec![serenity::CreateActionRow::Buttons(
                    map_search
                        .iter()
                        .enumerate()
                        .map(|(i, mapname)| {
                            CreateButton::new(format!("{uuid}.{}", i))
                                .label(&mapname.name)
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .collect(),
                )];

                let embed = CreateEmbed::new().title("Which map?").fields(
                    (1..map_search_len + 1)
                        .into_iter()
                        .map(|i| (format!("{i}."), map_search[i - 1].name.clone(), false)),
                );

                CreateReply::default().embed(embed).components(components)
            };

            ctx.send(map_list_reply).await?;

            let mut picked_map = String::new();

            while let Some(prompt_response) = serenity::ComponentInteractionCollector::new(ctx)
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(Duration::from_secs(10))
                .filter(move |response| {
                    let possible_buttons: Vec<String> =
                        (0..map_search_len).map(|e| format!("{uuid}.{e}")).collect();

                    possible_buttons.contains(&response.data.custom_id)
                })
                .await
            {
                let map_search_vec: Vec<String> = map_search
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| prompt_response.data.custom_id == format!("{uuid}.{i}"))
                    .map(|(_, e)| e.name.clone())
                    .collect();

                picked_map = map_search_vec[0].clone();

                prompt_response
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;

                prompt_response.message.delete(ctx).await?;

                break;
            }

            picked_map
        } else {
            map_search[0].name.clone()
        }
    };

    let res = reqwest::get(format!("https://tempus2.xyz/api/v0/maps/name/{map_name}/zones/typeindex/map/1/records/player/{tempus_id}/{}", class as usize)).await?.text().await?;

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
