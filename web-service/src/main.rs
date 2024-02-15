
use axum::{
    routing::get,
    Router,
};
use nervo_bot_app::nervo_app;
use qdrant_client::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build our application with a single route
    //let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //axum::serve(listener, app).await.unwrap();

    //nervo_app::start_nervo_bot().await?

    let qdrant_client = make_client().await?;
    //qdrant_client.create_collection("test_collection").await?;
    let collections_list = qdrant_client.list_collections().await?;
    println!("{:?}", collections_list);

    Ok(())
}

async fn make_client() -> anyhow::Result<QdrantClient> {
    let client = QdrantClient::from_url("https://e70f67ce-3a42-40a2-ae33-6426722829f4.us-east4-0.gcp.cloud.qdrant.io:6334")
        // using an env variable for the API KEY for example
        .with_api_key("gRObXeid0W-pUdT-XN6uWejrPM9jtPMtTvl6MzKZnA50IGfgYcw3Ug")
        .build()?;
    Ok(client)
}