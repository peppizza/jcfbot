mod commands;
mod consts;

use commands::{tempus::link::link, voteboil::voteboil};
use rand::seq::IndexedRandom;

use std::{env, sync::Arc, time::Duration};

use anyhow::Error;
use poise::serenity_prelude as serenity;
use tracing::{error, info};

use crate::{
    commands::tempus::{
        ranks::rank,
        shutdown,
        times::{dbtime, dctime, dtime, dttime, sbtime, sctime, stime, sttime},
    },
    consts::MAGIC_EIGHT_BALL,
};

type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            if let Err(e) = ctx.reply(":(").await {
                error!("we're really in the shit now {:?}", e);
            };

            error!("Error in command `{}` : {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().unwrap();

    let options = poise::FrameworkOptions {
        commands: vec![
            voteboil(),
            link(),
            stime(),
            dtime(),
            sbtime(),
            dbtime(),
            sctime(),
            dctime(),
            sttime(),
            dttime(),
            shutdown(),
            rank(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),

        event_handler: |ctx, event, _framework, _data| {
            Box::pin(async move {
                info!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );

                match event {
                    serenity::FullEvent::Ready { data_about_bot, .. } => {
                        info!("Logged in as: {}", data_about_bot.user.name);
                    }
                    serenity::FullEvent::Message { new_message } => {
                        if new_message.mentions_me(ctx).await?
                            && new_message.content.contains("is this true")
                            && new_message.author.id != ctx.cache.current_user().id
                        {
                            let answer = {
                                let mut rng = rand::rng();
                                MAGIC_EIGHT_BALL.choose(&mut rng).unwrap()
                            };

                            new_message.reply_ping(ctx, answer.to_string()).await?;
                        }
                    }
                    _ => {}
                };

                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shard_manager.shutdown_all().await;
    });

    client.start().await.unwrap();
}
