// SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
//
// SPDX-License-Identifier: AGPL-3.0-only

use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::info;
use oxhttp::Client;
use oxhttp::model::{Request, Method, Status};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<ConnectEvent>) -> Result<Value, Error> {
    let login_url = env::var("LOGIN_URL")?;
    info!("login_url = {}", login_url);

    let user_id_key = env::var("USER_ID_KEY")?;
    info!("user_id_key = {}", user_id_key);

    let voter_pin_key = env::var("VOTER_PIN_KEY")?;
    info!("voter_pin_key = {}", voter_pin_key);

    let (connect_event, _) = event.into_parts();
    let user_id_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterUserId")
        .unwrap();
    info!("user_id_value = {}", user_id_value);
    let voter_pin_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterPIN")
        .unwrap();
    info!("voter_pin_value = {}", voter_pin_value);

    let mut data = HashMap::new();

    data.insert(user_id_key, user_id_value);
    data.insert(voter_pin_key, voter_pin_value);

    let client = Client::new();
    let response = client.request(
        Request::builder(
            Method::POST,
            login_url.parse()?
        )
        .with_body(serde_json::to_string(&data)?)
    )?;
    
    assert_eq!(response.status(), Status::OK);

    let body = response.into_body().to_string()?;
    let body_value: Value = serde_json::from_str(&body)?;

    // Extract some useful information from the request
    Ok(json!({
        "AuthToken": body_value["auth-token"]
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

    info!("starting up authenticate_voter lambda");

    run(service_fn(function_handler)).await
}
