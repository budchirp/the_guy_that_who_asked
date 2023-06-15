mod commands;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::Activity;
use serenity::prelude::*;

use rand::{thread_rng, Rng};

use tokio::sync::mpsc::channel;
use tokio::time::sleep;

use tracing::{error, info};

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_lowercase() == "who asked" {
            info!("who asked");

            if let Err(why) = msg.channel_id.say(&ctx.http, "me").await {
                error!("Error while sending message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            info!("Recieved command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                info!("Cannot response to slash command: {}", why);
            }
        };
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID!")
                .parse()
                .expect("GUILD_ID must be integer"),
        );

        let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::ping::register(command))
        })
        .await;

        let ctx = Arc::new(ctx);

        let texts = vec!["L", "ratio"];

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let (tx, mut rx) = channel::<String>(1);

            tokio::spawn(async move {
                loop {
                    let idx = thread_rng().gen_range(0..texts.len());
                    let message = texts[idx].to_string();

                    if let Err(_) = tx.send(message).await {
                        break;
                    }
                    sleep(Duration::from_secs(2 * 60)).await;
                }
            });

            while let Some(message) = rx.recv().await {
                info!("Set activity as `{}`", message);
                ctx.set_activity(Activity::watching(message)).await;
            }

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file!");
    tracing_subscriber::fmt::init();

    let token = env::var("TOKEN").expect("Expected TOKEN");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error starting client!");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
