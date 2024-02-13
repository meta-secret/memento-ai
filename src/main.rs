mod ai;
mod common;
mod telegram;
mod nervo_app;

///https://jason5lee.me/2022/04/12/telegram-bot-rust-azure-function/
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    nervo_app::start_nervo_bot().await?
}
