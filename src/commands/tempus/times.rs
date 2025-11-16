use std::{fmt::Display, time::Duration};

use ::serenity::all::{CreateButton, CreateEmbed, CreateInteractionResponse, Mentionable};
use hhmmss::Hhmmss;
use poise::{CreateReply, command, serenity_prelude as serenity};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};

use crate::{
    Context,
    commands::tempus::{get_tempus_id, link::TempusPlayerInfo},
};

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn stime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, &map, Classes::Soldier, ZoneType::Map, 1).await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dtime(ctx: Context<'_>, map: String) -> Result<(), anyhow::Error> {
    time(ctx, &map, Classes::Demoman, ZoneType::Map, 1).await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn sbtime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Soldier,
        ZoneType::Bonus,
        zone_index.unwrap_or(1),
    )
    .await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dbtime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Demoman,
        ZoneType::Bonus,
        zone_index.unwrap_or(1),
    )
    .await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn sctime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Soldier,
        ZoneType::Course,
        zone_index.unwrap_or(1),
    )
    .await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dctime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Demoman,
        ZoneType::Course,
        zone_index.unwrap_or(1),
    )
    .await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn sttime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Soldier,
        ZoneType::Trick,
        zone_index.unwrap_or(1),
    )
    .await
}

#[command(prefix_command, global_cooldown = 2, user_cooldown = 5)]
pub async fn dttime(
    ctx: Context<'_>,
    map: String,
    zone_index: Option<u64>,
) -> Result<(), anyhow::Error> {
    time(
        ctx,
        &map,
        Classes::Demoman,
        ZoneType::Trick,
        zone_index.unwrap_or(1),
    )
    .await
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
pub enum Classes {
    Overall,
    Soldier = 3,
    Demoman = 4,
}

impl Display for Classes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ZoneType {
    Map,
    Course,
    Bonus,
    Trick,
}

impl Display for ZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

async fn time(
    ctx: Context<'_>,
    map: &str,
    class: Classes,
    zone_type: ZoneType,
    zone_index: u64,
) -> Result<(), anyhow::Error> {
    let uuid = ctx.id();
    let discord_id = ctx.author().id.get() as i64;

    let tempus_id = match get_tempus_id(discord_id).await {
        Ok(id) => id,
        Err(e) => {
            ctx.reply(format!("{}", e)).await?;
            return Ok(());
        }
    };

    let search = reqwest::get(format!(
        "https://tempus2.xyz/api/v0/search/playersAndMaps/{map}"
    ))
    .await?;

    if search.status() == StatusCode::TOO_MANY_REQUESTS {
        ctx.send(poise::CreateReply::default().content(format!(
            "{} ratelimited!!!! {}",
            ctx.author_member().await.unwrap().mention(),
            serenity::UserId::new(253290704384557057).mention()
        )))
        .await?;
        ctx.framework().shard_manager().shutdown_all().await;
        return Ok(());
    }

    let map_search: TempusSearchResult = serde_json::from_str(&search.text().await?)?;
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

    let res = reqwest::get(format!("https://tempus2.xyz/api/v0/maps/name/{map_name}/zones/typeindex/{}/{zone_index}/records/player/{tempus_id}/{}", zone_type, class as usize)).await?;

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

    let completion_data: TempusCompletionData = serde_json::from_str(&res.text().await?)?;
    let completions = if class == Classes::Soldier {
        completion_data.completion_info.soldier
    } else {
        completion_data.completion_info.demoman
    };

    if let Some(completion_data) = completion_data.result {
        let time = Duration::from_secs_f64(completion_data.duration);

        if zone_type == ZoneType::Map {
            ctx.reply(format!(
                "{} is ranked {}/{} on {}, {}",
                completion_data.player_info.name,
                completion_data.rank,
                completions,
                map_name,
                time.hhmmssxxx()
            ))
            .await?;
        } else {
            ctx.reply(format!(
                "{} is ranked {}/{} on {} {} {}, {}",
                completion_data.player_info.name,
                completion_data.rank,
                completions,
                map_name,
                zone_type,
                zone_index,
                time.hhmmssxxx()
            ))
            .await?;
        }
    } else {
        ctx.reply(format!("No {class} completion on {map} :("))
            .await?;
    }

    Ok(())
}
