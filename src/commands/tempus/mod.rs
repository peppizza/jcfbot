use crate::Context;
use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use sqlx::{Connection, SqliteConnection};
use std::{env, fmt::Display};

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

pub enum TempusPlacement {
    Record,
    TopTime(i64),
    G1,
    G2,
    G3,
    G4,
    G5,
}

impl Display for TempusPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TempusPlacement::Record => write!(f, "WR"),
            TempusPlacement::TopTime(p) => write!(f, "TopTime({p})"),
            TempusPlacement::G1 => write!(f, "Group 1"),
            TempusPlacement::G2 => write!(f, "Group 2"),
            TempusPlacement::G3 => write!(f, "Group 3"),
            TempusPlacement::G4 => write!(f, "Group 4"),
            TempusPlacement::G5 => write!(f, "Group 5"),
        }
    }
}

pub fn calculate_placement(rank: i64, total_completions: f32) -> TempusPlacement {
    let g1 = ((total_completions - 10_f32).ceil() * 0.02 + 10_f32).min(30_f32);
    let g2 = ((total_completions - 10_f32).ceil() * 0.05 + g1).min(80_f32);
    let g3 = ((total_completions - 10_f32).ceil() * 0.125 + g2).min(205_f32);
    let g4 = ((total_completions - 10_f32).ceil() * 0.333 + g3).min(539_f32);

    if rank == 1 {
        TempusPlacement::Record
    } else if (2..=10).contains(&rank) {
        TempusPlacement::TopTime(rank)
    } else if rank as f32 <= g1 {
        TempusPlacement::G1
    } else if rank as f32 <= g2 {
        TempusPlacement::G2
    } else if rank as f32 <= g3 {
        TempusPlacement::G3
    } else if rank as f32 <= g4 {
        TempusPlacement::G4
    } else {
        TempusPlacement::G5
    }
}
