use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use reqwest;
use std::collections::HashMap;
use serde_json::{json, Value};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<ConnectEvent>) -> Result<Value, Error> {
    let login_url = env::var("LOGIN_URL")?;
    let user_id_key = env::var("USER_ID_KEY")?;
    let voter_pin_key = env::var("VOTER_PIN_KEY")?;
    let (connect_event, _) = event.into_parts();
    let user_id_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterUserId")
        .unwrap();
    let voter_pin_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterPIN")
        .unwrap();

    let mut data = HashMap::new();

    data.insert(user_id_key, user_id_value);
    data.insert(voter_pin_key, voter_pin_value);

    let client = reqwest::Client::new();
    let response = client.post(login_url)
            .json(&data)
            .send()
            .await?;

    // Extract some useful information from the request
    Ok(json!({
        "statusCode": response.status().as_str(),
        "body": response.text().await?
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
