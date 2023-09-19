mod commands;

use std::env;

use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_lowercase() == "who asked" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "me").await {
                eprintln!("Error while sending message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                _ => "Not implemented!".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                eprintln!("Cannot response to slash command: {}", why);
            }
        };
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _commands = Command::create_global_application_command(&ctx.http, |command| {
            commands::ping::register(command)
        })
        .await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file!");

    let token = env::var("TOKEN").expect("Expected TOKEN");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error starting client!");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
