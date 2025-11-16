use crate::Context;

use ::serenity::all::{
    CreateAttachment, CreateInteractionResponse, EditMessage, Mentionable, UserId,
};
use anyhow::{Error, anyhow};
use magick_rust::{MagickWand, magick_wand_genesis};
use poise::{CreateReply, serenity_prelude as serenity};
use rand::Rng;
use std::{
    sync::{Arc, Once},
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::info;

static START: Once = Once::new();

#[poise::command(prefix_command)]
pub async fn voteboil(
    ctx: Context<'_>,
    #[description = "who to boil"] target: serenity::Member,
) -> Result<(), Error> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    if target.user.id == 1439122209170784386 {
        ctx.reply(">:(").await?;
        return Ok(());
    };

    let uuid = ctx.id();

    let target_name = target.display_name();
    let target_pfp = target.face();
    let author = ctx.author_member().await.unwrap();

    let yes_votes = Arc::new(RwLock::new(vec![author.user.id]));
    let no_votes: Arc<RwLock<Vec<UserId>>> = Arc::new(RwLock::new(vec![]));

    let (embed, components) = voteboil_embed_and_component(
        ctx,
        target_name,
        &target_pfp,
        yes_votes.clone(),
        no_votes.clone(),
        uuid,
    )
    .await;

    info!("yes: {:?}, no: {:?}", yes_votes, no_votes);
    let vote_msg = ctx
        .send(CreateReply::default().embed(embed).components(components))
        .await?;

    while let Some(vote) = serenity::ComponentInteractionCollector::new(ctx)
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(15))
        .filter(move |vote| {
            vote.data.custom_id == format!("{uuid}.yes")
                || vote.data.custom_id == format!("{uuid}.no")
        })
        .await
    {
        let voter_id = vote.user.id;

        if yes_votes.read().await.contains(&voter_id) || no_votes.read().await.contains(&voter_id) {
            vote.create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("erm")
                        .ephemeral(true),
                ),
            )
            .await?;

            continue;
        };

        if vote.data.custom_id == format!("{uuid}.yes") {
            yes_votes.write().await.push(voter_id);
        } else {
            no_votes.write().await.push(voter_id);
        }

        info!("yes: {:?}, no: {:?}", yes_votes, no_votes);

        let mut msg = vote.message.clone();

        let (embed, components) = voteboil_embed_and_component(
            ctx,
            target_name,
            &target_pfp,
            yes_votes.clone(),
            no_votes.clone(),
            uuid,
        )
        .await;

        msg.edit(
            ctx,
            EditMessage::default().embed(embed).components(components),
        )
        .await?;

        vote.create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;
    }

    let (embed, _) = voteboil_embed_and_component(
        ctx,
        target_name,
        &target_pfp,
        yes_votes.clone(),
        no_votes.clone(),
        uuid,
    )
    .await;
    vote_msg
        .edit(ctx, CreateReply::default().embed(embed).components(vec![]))
        .await?;

    if yes_votes.read().await.len() >= no_votes.read().await.len() {
        let img = boil_image(&target_pfp).await?;

        ctx.send(
            CreateReply::default()
                .attachment(CreateAttachment::bytes(img, "boiled.jpg"))
                .content(format!("{} HAS BEEN BOILED", target.mention())),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default().content(format!("{} has been spared...", target.mention())),
        )
        .await?;
    }

    Ok(())
}

async fn boil_image(target_pfp: &str) -> Result<Vec<u8>, anyhow::Error> {
    let stewchoice = {
        let mut rng = rand::rng();
        rng.random_range(1..8)
    };

    let stewwand = MagickWand::new();
    let pfpwand = MagickWand::new();

    let stewimg = tokio::fs::read(format!("stews/{stewchoice}.jpg")).await?;
    let pfpimg = reqwest::get(target_pfp).await?.bytes().await?;

    stewwand.read_image_blob(stewimg)?;
    pfpwand.read_image_blob(pfpimg)?;

    pfpwand.adaptive_resize_image(556, 556)?;

    stewwand.compose_images_gravity(
        &pfpwand,
        magick_rust::CompositeOperator::Atop,
        magick_rust::GravityType::Center,
    )?;

    stewwand
        .write_image_blob("jpeg")
        .map_err(|e| anyhow!("{e}"))
}

async fn voteboil_embed_and_component(
    ctx: Context<'_>,
    target_name: &str,
    target_pfp: &str,
    yes_votes: Arc<RwLock<Vec<UserId>>>,
    no_votes: Arc<RwLock<Vec<UserId>>>,
    uuid: u64,
) -> (serenity::CreateEmbed, Vec<serenity::CreateActionRow>) {
    let guild = ctx.guild().unwrap().clone();

    let mut yes_voter_names: Vec<String> = vec![];
    let mut no_voter_names: Vec<String> = vec![];

    for yes_voter in yes_votes.read().await.iter() {
        yes_voter_names.push(
            guild
                .member(ctx, yes_voter)
                .await
                .unwrap()
                .display_name()
                .to_string(),
        );
    }

    for no_voter in no_votes.read().await.iter() {
        no_voter_names.push(
            guild
                .member(ctx, no_voter)
                .await
                .unwrap()
                .display_name()
                .to_string(),
        );
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("DECIDE {}'S FATE", target_name.to_uppercase()))
        .image(target_pfp)
        .field("YES VOTES", yes_voter_names.join("\n"), true)
        .field("NO VOTES", no_voter_names.join("\n"), true);

    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("{uuid}.yes"))
            .label("YES!!!")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("{uuid}.no"))
            .label("no")
            .style(serenity::ButtonStyle::Danger),
    ])];

    (embed, components)
}
