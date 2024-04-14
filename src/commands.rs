use crate::calendar::get_available_time_slots;
use crate::routes::{get_service, save_appointment};
use actix_web::web;

use sqlx::PgPool;

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveUserName,
    ReceiveService {
        user_name: String,
    },
    ReceiveDateTime {
        user_name: String,
        service_name: String,
    },
    AppointmentConfirmed {
        user_name: String,
        service_name: String,
        appointment_time: String,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Help,
    Start,
    Cancel,
}

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveUserName].endpoint(receive_user_name))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::ReceiveService { user_name }].endpoint(receive_service))
        .branch(
            case![State::ReceiveDateTime {
                user_name,
                service_name
            }]
            .endpoint(receive_data_time),
        )
        .branch(
            case![State::AppointmentConfirmed {
                user_name,
                service_name,
                appointment_time
            }]
            .endpoint(appointment_confirmation),
        );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's get started! What's your name?")
        .await?;
    dialogue.update(State::ReceiveUserName).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancel the dialog.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to process the message. Dial /n 
    help for information about commands.",
    )
    .await?;
    Ok(())
}
async fn receive_user_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    db_pool: web::Data<PgPool>,
) -> HandlerResult {
    if let Some(user_name) = msg.text() {
        let services = get_service(db_pool).await.expect("services");

        let buttons = services
            .iter()
            .map(|service| {
                vec![InlineKeyboardButton::callback(
                    format!("{}", service.service_name,),
                    format!("{}_{}", service.service_id, service.service_name),
                )]
            })
            .collect::<Vec<_>>();

        bot.send_message(msg.chat.id, "Select service:")
            .reply_markup(InlineKeyboardMarkup::new(buttons))
            .await?;
        dialogue
            .update(State::ReceiveService {
                user_name: user_name.to_owned(),
            })
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Please enter your first name:")
            .await?;
    }
    Ok(())
}

async fn receive_service(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    db_pool: web::Data<PgPool>,
    user_name: String,
) -> HandlerResult {
    if let Some(callback_data) = q.data.as_ref() {
        if let Some(message) = q.message.as_ref() {
            let chat_id = message.chat.id;
            log::info!(
                "User {} selected service with callback data: {}",
                user_name,
                callback_data
            );

            bot.answer_callback_query(&q.id).await?;

            let parts: Vec<&str> = callback_data.splitn(2, '_').collect();
            if parts.len() == 2 {
                let service_id: i64 = parts[0].parse().expect("Failed to parse service_id");
                let service_name = parts[1].to_string();

                let available_slots = get_available_time_slots(db_pool, service_id)
                    .await
                    .expect("Failed to fetch available time slots");

                let buttons = available_slots
                    .iter()
                    .map(|slot| {
                        vec![InlineKeyboardButton::callback(
                            slot.to_string(),
                            format!("{}", slot),
                        )]
                    })
                    .collect::<Vec<_>>();

                bot.send_message(chat_id, "Select the date and time:")
                    .reply_markup(InlineKeyboardMarkup::new(buttons))
                    .await?;

                dialogue
                    .update(State::ReceiveDateTime {
                        user_name,
                        service_name: service_name.to_string(),
                    })
                    .await?;
            } else {
                log::error!("Callback data is in an incorrect format: {}", callback_data);
            }
        } else {
            log::error!("Message not found in callback query");
        }
    } else {
        log::error!("Callback data not found");
    }

    Ok(())
}

async fn receive_data_time(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Ok(Some(State::ReceiveDateTime {
        user_name,
        service_name,
    })) = dialogue.get().await
    {
        if let Some(time_slot) = q.data.as_ref() {
            if let Some(message) = q.message.as_ref() {
                let chat_id = message.chat.id;

                bot.send_message(
                    chat_id,
                    format!(
                        "😎Dear {user_name}❗\n
                Your appointment for ➡️ {service_name} \n
                at ➡️ {time_slot}' \n
                ✅ Has been successfully booked!"
                    ),
                )
                .reply_markup(InlineKeyboardMarkup::new(vec![
                    vec![InlineKeyboardButton::callback("Yes, confirm", "confirm")],
                    vec![InlineKeyboardButton::callback("No, change", "cancel")],
                ]))
                .await?;

                dialogue
                    .update(State::AppointmentConfirmed {
                        user_name,
                        service_name,
                        appointment_time: time_slot.to_string(),
                    })
                    .await?;
            } else {
                log::error!("CallbackQuery without a message received: {:?}", q);
            }
        }
    } else {
        log::error!(
            "Could not find information about your previous operation: {:?}",
            q
        );
    }
    Ok(())
}

async fn appointment_confirmation(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    db_pool: web::Data<PgPool>,
) -> HandlerResult {
    if let Ok(Some(State::AppointmentConfirmed {
        user_name,
        service_name,
        appointment_time,
    })) = dialogue.get().await
    {
        save_appointment(db_pool, user_name, service_name, appointment_time)
            .await
            .expect("Error saving a record");

        if let Some(message) = q.message {
            bot.send_message(
                message.chat.id,
                "🎉 Your entry has been confirmed. \n
                     ❤️ Thank you for using our services",
            )
            .await?;
        }
        dialogue.exit().await?;
        Ok(())
    } else {
        Err("Unable to get AppointmentConfirmed status".into())
    }
}
