use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use tg_bot::commands::{schema, State};
use tg_bot::configuration::get_configuration;
use tg_bot::startup::{get_connection_pool, Application};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let bot = Bot::new("TOKEN_BOT");

    let webhook_url = format!("https://0000-000-000-000-00.ngrok-free.app/TOKEN_BOT")
        .parse()
        .unwrap();
    bot.set_webhook(webhook_url)
        .await
        .expect("Failed to set webhook");

    let server_future = async {
        let application = Application::build(configuration.clone()).await?;
        application.run_until_stopped().await
    };

    let con_pool = actix_web::web::Data::new(get_connection_pool(&configuration.database));

    let storage = InMemStorage::<State>::new();

    let bot_future = async move {
        Dispatcher::builder(bot, schema())
            .dependencies(dptree::deps![storage, con_pool])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    };

    tokio::join!(server_future, bot_future);
    Ok(())
}
