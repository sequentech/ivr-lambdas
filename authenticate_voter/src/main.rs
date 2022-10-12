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

    match status {
        Status::OK => {
            let ret_value = json!({
                "AuthToken": body_value["auth-token"]
            });
            event!(Level::DEBUG, ret_value = ret_value.to_string());
        
            // Return the auth_token
            Ok(ret_value)
        },
        _ => Err("invalid-status".into())
    }    
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
    use serde_json::Value;
    use tokio;
    use lambda_runtime::Error;
    use serial_test::serial;
    use aws_lambda_events::event::connect::ConnectEvent;
    use httpmock::prelude::*;
    use httpmock::Mock;
    use crate::function_handler;

    // Set environment variables. If any of the values is an empty string,
    // unsets the variable. 
    //
    // IMPORTANT: env vars are set for the whole executable, so changing this
    // might create run conditions on any function that depends on env
    // variables.
    fn set_env_vars(env_vars: &HashMap<&str, &str>) {
        for (env_var_name, env_var_value) in env_vars.iter() {
            if env_var_value.len() > 0 {
                env::set_var(env_var_name, env_var_value);
            } else {
                env::remove_var(env_var_name);
            }
        }
    }

    // default init function for unit tests
    fn init<'a>(
        server: &'a MockServer,
        override_env_vars: Option<HashMap<&str, &str>>,
        auth_voter_path: Option<&str>,
        response: &str
    ) -> Mock<'a>
    {
        // Create a mock on the server.
        let auth_voter_path = auth_voter_path
            .unwrap_or("/authentication-success");
        let login_url = server.base_url() + auth_voter_path;
        let auth_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/authentication-success");
            then.status(200)
                .header("content-type", "application/json")
                .body(response);
        });

        let default_env_vars: HashMap<&str, &str> = HashMap::from([
            ("TRACING_LEVEL", "debug"),
            ("USER_ID_KEY", "user-id"),
            ("VOTER_PIN_KEY", "code"),
            ("LOGIN_URL", login_url.as_str())
        ]);
        let override_env_vars_val = override_env_vars
            .unwrap_or(Default::default());
        let env_vars: HashMap<&str, &str> = default_env_vars
            .into_iter()
            .chain(override_env_vars_val)
            .collect();
        set_env_vars(&env_vars);

        return auth_mock;
    }

    // calls the crate's lambda
    async fn call_lambda(connect_event_str: &str) -> Result<Value, Error> {
        let input: ConnectEvent = serde_json::from_str(connect_event_str)?;
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);
        let (connect_event, _) = event.clone().into_parts();
        println!(
            "connect_event = {}", 
            serde_json::to_string(&connect_event).unwrap_or(Default::default())
        );
        let event_result = function_handler(event)
            .await;
        return event_result;
    }

    // using #[tokio::test] instead of just #[test] to be able to call async
    // function in the test
    #[tokio::test]
    // we apply serial because we are changing the env vars for the whole
    // executable and other test function would do the same, so we need to
    // prevent a run condition
    #[serial]
    // Simulates how an authentication success should happen
    async fn authentication_success() {
        let server = MockServer::start();
        let auth_mock = init(
            &server,
            Default::default(),
            None,
            include_str!("../test/mock_backend/authentication_success.json")
        );

        let event_result = call_lambda(include_str!("../test/test_data_1.json"))
            .await
            .expect("failed to handle event");

        auth_mock.assert();
        assert_eq!(event_result["AuthToken"], "mock-token");
    }

    // simulates an authentication failure
    #[tokio::test]
    #[serial]
    async fn authentication_failure() {
        let server = MockServer::start();
        let auth_voter_path = "/authenticate-failure";
        init(
            &server,
            Default::default(),
            Some(auth_voter_path),
            include_str!(
                "../test/mock_backend/authentication_success.json"
            )
        );
        let auth_error_mock = server.mock(|when, then| {
            when.method(POST)
                .path(auth_voter_path);
            then.status(400)
                .header("content-type", "application/json")
                .body(include_str!(
                    "../test/mock_backend/authentication_failure.json"
                ));
        });
        let event_result = call_lambda(include_str!("../test/test_data_1.json"))
            .await;
        auth_error_mock.assert();
        event_result
            .expect_err("authentication succeeded when it should have failed");
    }

    // should panic with LOGIN_URL env var not set
    #[tokio::test]
    #[should_panic]
    #[serial]
    async fn unset_login_url_env_var() {
        let server = MockServer::start();
        init(
            &server,
            Some(HashMap::from([
                ("LOGIN_URL", "")
            ])),
            None,
            include_str!("../test/mock_backend/authentication_success.json")
        );
        call_lambda(include_str!("../test/test_data_1.json"))
            .await
            .expect("failed to handle event");
    }

    // should panic with VOTER_PIN_KEY env var not set
    #[tokio::test]
    #[should_panic]
    #[serial]
    async fn unset_voter_pin_key_env_var() {
        let server = MockServer::start();
        init(
            &server,
            Some(HashMap::from([
                ("VOTER_PIN_KEY", "")
            ])),
            None,
            include_str!("../test/mock_backend/authentication_success.json")
        );
        call_lambda(include_str!("../test/test_data_1.json"))
            .await
            .expect("failed to handle event");
    }

    // should panic with USER_ID_KEY env var not set
    #[tokio::test]
    #[should_panic]
    #[serial]
    async fn unset_user_id_key_env_var() {
        let server = MockServer::start();
        init(
            &server,
            Some(HashMap::from([
                ("USER_ID_KEY", "")
            ])),
            None,
            include_str!("../test/mock_backend/authentication_success.json")
        );
        call_lambda(include_str!("../test/test_data_1.json"))
            .await
            .expect("failed to handle event");
    }
}
