use nervo_bot_core::nervo_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Start Nervo Bot");

    // build our application with a single route
    //let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //axum::serve(listener, app).await.unwrap();

    nervo_app::start_nervo_bot().await?;
    Ok(())
}
