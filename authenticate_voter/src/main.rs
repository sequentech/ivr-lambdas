// SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
//
// SPDX-License-Identifier: AGPL-3.0-only

use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{event, Level};
use oxhttp::Client;
use oxhttp::model::{Request, Method, Status};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<ConnectEvent>)
    -> Result<Value, Error>
{
    let (connect_event, connect_context) = event.into_parts();
    event!(
        Level::DEBUG,
        connect_event = serde_json::to_string(&connect_event)?,
        connect_context = serde_json::to_string(&connect_context)?
    );

    // Example base_url: 
    // https://clientname.example.com/iam/api/auth-event/{{election_id}}/authenticate/
    // Note that {{election_id}} will be substituted with the election id
    let base_url = env::var("BASE_URL")?;
    event!(Level::INFO, base_url);

    // This is the authentication extra field name for the user id
    let user_id_key = env::var("USER_ID_KEY")?;
    event!(Level::INFO, user_id_key);

    // This is the authentication extra field name for the voter pin
    let voter_pin_key = env::var("VOTER_PIN_KEY")?;
    event!(Level::INFO, voter_pin_key);

    let election_id: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("ElectionId")
        .ok_or(String::from("ElectionId contact data attribute missing"))?;
    event!(Level::INFO, election_id);

    let user_id_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterUserId")
        .ok_or(String::from("VoterUserId contact data attribute missing"))?;
    event!(Level::INFO, user_id_value);
    let voter_pin_value: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("VoterPIN")
        .ok_or(String::from("VoterPIN contact data attribute missing"))?;
    event!(Level::DEBUG, voter_pin_value);

    let mut data = HashMap::new();
    data.insert(user_id_key, user_id_value);
    data.insert(voter_pin_key, voter_pin_value);
    let body: String = serde_json::to_string(&data)?;

    let client = Client::new();
    let login_url = base_url.replace("{{election_id}}", election_id);
    event!(Level::DEBUG, request_url = login_url, request_body = body);
    let response = client.request(
        Request::builder(
            Method::POST,
            login_url.parse()?
        )
        .with_body(body)
    )?;

    let status = response.status();
    event!(Level::INFO, request_response_status = status.to_string());

    let body = response.into_body().to_string()?;
    event!(Level::DEBUG, request_response_body = body);
    let body_value: Value = serde_json::from_str(&body)?;

    match status {
        Status::OK => {
            let vote_permission_token: &Value = &body_value["vote-permission-token"];
            event!(Level::DEBUG, "vote_permission_token={:?}", vote_permission_token);
            let vote_children_info: &Value = &body_value["vote-children-info"];
            event!(Level::DEBUG, "vote_children_info={:?}", vote_children_info);

            if vote_permission_token.is_string()
                && vote_permission_token.to_string().len() > 0
            {
                let ret_value = json!({
                    "AuthToken": vote_permission_token,
                    "ElectionId": election_id
                });
                event!(Level::DEBUG, ret_value = ret_value.to_string());

                // Return the vote_permission_token
                Ok(ret_value)
            }
            else if vote_children_info.is_array()
                && vote_children_info.as_array().unwrap().len() > 0
                && vote_children_info.as_array().unwrap()[0].is_object()
            {
                let child_election = vote_children_info
                    .as_array()
                    .unwrap()[0]
                    .as_object()
                    .unwrap();
                // we perform login only to the first child election
                let ret_value = json!({
                    "AuthToken": child_election["vote-permission-token"]
                        .to_string(),
                    "ElectionId": child_election["auth-event-id"].to_string()
                });
                event!(Level::DEBUG, ret_value = ret_value.to_string());

                // Return the vote_permission_token
                Ok(ret_value)
            } else {
                Err("empty-vote-permission-token".into())
            }
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
    use serde_json::json;
    use tokio;
    use lambda_runtime::Error;
    use serial_test::serial;
    use aws_lambda_events::event::connect::ConnectEvent;
    use httpmock::prelude::*;
    use httpmock::Mock;
    //use num_bigint::BigUint;
    //use num_traits::Num;

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
        let base_url = server.base_url() + auth_voter_path;
        let auth_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/authentication-success")
                .json_body(json!({ "user-id": "100", "code": "22345678" }));
            then.status(200)
                .header("content-type", "application/json")
                .body(response);
        });

        let default_env_vars: HashMap<&str, &str> = HashMap::from([
            ("TRACING_LEVEL", "debug"),
            ("USER_ID_KEY", "user-id"),
            ("VOTER_PIN_KEY", "code"),
            ("BASE_URL", base_url.as_str())
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

    //#[test]
    //fn testing_biguint() {
    //    let hash = String::from("27d9e601718d704671ab3c3dfcf7fd1dcc329ba2b69fe5e443469beef0ea9bdc");
    //    let is = BigUint::from_str_radix(hash.as_str(), 16).unwrap();
    //    let should = BigUint::from_bytes_le(
    //         String::from("18025194348382480456338733710662541073828462113497433353157482543816263769052").as_bytes()
    //    );
    //    assert_eq!(is, should);
    //}

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

        println!(
            "event_result = {:?}", 
            &event_result
        );
        auth_mock.assert();
        assert_eq!(event_result["AuthToken"], "khmac:///sha-256;c4ba96310ea7474b4ee2e84b00eaf412786816ea7d3713af866dab67c3201668/4cf53604330bab6a6179de2e:AuthEvent:17:vote:1665653516");
    }

    // simulates an authentication failure because input data is invalid
    #[tokio::test]
    #[serial]
    async fn authentication_failure1() {
        let server = MockServer::start();
        let auth_mock = init(
            &server,
            Default::default(),
            None,
            include_str!("../test/mock_backend/authentication_success.json")
        );

        call_lambda(include_str!("../test/test_data_2.json"))
            .await
            .expect_err("authentication succeeded when it should have failed");

        auth_mock.assert_hits(0);
    }

    // simulates an authentication failure (independent of incoming data)
    #[tokio::test]
    #[serial]
    async fn authentication_failure2() {
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

    // should panic with BASE_URL env var not set
    #[tokio::test]
    #[should_panic]
    #[serial]
    async fn unset_base_url_env_var() {
        let server = MockServer::start();
        init(
            &server,
            Some(HashMap::from([
                ("BASE_URL", "")
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
