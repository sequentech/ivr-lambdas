// SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
//
// SPDX-License-Identifier: AGPL-3.0-only

use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{event, Level, instrument};
use oxhttp::Client;
use oxhttp::model::{Request, Method, Status};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
#[instrument]
async fn function_handler(event: LambdaEvent<ConnectEvent>)
    -> Result<Value, Error>
{
    let (connect_event, connect_context) = event.into_parts();
    event!(
        Level::DEBUG,
        connect_event = serde_json::to_string(&connect_event)?,
        connect_context = serde_json::to_string(&connect_context)?
    );

    let login_url = env::var("LOGIN_URL")?;
    event!(Level::INFO, login_url);

    let user_id_key = env::var("USER_ID_KEY")?;
    event!(Level::INFO, user_id_key);

    let voter_pin_key = env::var("VOTER_PIN_KEY")?;
    event!(Level::INFO, voter_pin_key);

    let user_id_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterUserId")
        .unwrap();
    event!(Level::INFO, user_id_value);
    let voter_pin_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterPIN")
        .unwrap();
    event!(Level::DEBUG, voter_pin_value);

    let mut data = HashMap::new();
    data.insert(user_id_key, user_id_value);
    data.insert(voter_pin_key, voter_pin_value);
    let body: String = serde_json::to_string(&data)?;

    let client = Client::new();
    event!(Level::DEBUG, request_url = login_url, request_body = body);
    let response = client.request(
        Request::builder(
            Method::POST,
            login_url.parse()?
        )
        .with_body(body)
    )?;

    let status = response.status();
    event!(Level::INFO, status = status.to_string());

    let body = response.into_body().to_string()?;
    event!(Level::DEBUG, body);
    let body_value: Value = serde_json::from_str(&body)?;

    assert_eq!(status, Status::OK);
    let ret_value = json!({
        "AuthToken": body_value["auth-token"]
    });
    event!(Level::DEBUG, ret_value = ret_value.to_string());

    // Return the auth_token
    Ok(ret_value)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let tracing_level_str = env::var("TRACING_LEVEL")
        .unwrap_or(String::from("info"));
    let tracing_level: Level = Level::from_str(&tracing_level_str)?;

    tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion
        // time.
        .without_time()
        .init();

    event!(Level::INFO, tracing_level_str);
    event!(Level::INFO, "starting up `authenticate_voter` lambda");

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::include_str;
    use tokio;
    use aws_lambda_events::event::connect::ConnectEvent;
    use crate::function_handler;

    // Configures environment variables
    fn set_env_vars(env_vars: &HashMap<&str, &str>) {
        for (env_var_name, env_var_value) in env_vars.iter() {
            env::set_var(env_var_name, env_var_value);
        }
    }

    // default init function for unit tests
    fn init(override_env_vars: Option<HashMap<&str, &str>>) {
        let default_env_vars: HashMap<&str, &str> = HashMap::from([
            ("TRACING_LEVEL", "debug"),
            (
                "LOGIN_URL",
                "http://127.0.0.1:8000/authenticate_voter.response_200.json"
            ),
            ("USER_ID_KEY", "user-id"),
            ("VOTER_PIN_KEY", "code")
        ]);
        let override_env_vars_val = override_env_vars
            .unwrap_or(Default::default());
        let env_vars: HashMap<&str, &str> = default_env_vars
            .into_iter()
            .chain(override_env_vars_val)
            .collect();
        set_env_vars(&env_vars);
    }

    // using #[tokio::test] instead of just #[test] to be able to call async
    // function in the test
    #[tokio::test]
    async fn authentication_success() {
        init(None);

        let input: ConnectEvent = serde_json::from_str(
            include_str!("../test/test_data_1.json")
        ).unwrap();
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);
        let (connect_event, _) = event.clone().into_parts();
        println!(
            "connect_event = {}", 
            serde_json::to_string(&connect_event).unwrap_or(Default::default())
        );
        let event_result = function_handler(event)
            .await
            .expect("failed to handle event");
        assert_eq!(event_result["AuthToken"], "mock-token");
    }
}
