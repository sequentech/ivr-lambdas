// SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
//
// SPDX-License-Identifier: AGPL-3.0-only

extern crate chrono;
use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use sha2::{Sha256, Digest};
use std::env;
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use num_bigint::BigUint;
use strand::context::Ctx;
use strand::backend::num_bigint_sha2::{BigintCtx, P2048};
use strand::elgamal::PublicKey;
use chrono::prelude::*;
use tracing::{event, Level};

use oxhttp::Client;
use oxhttp::model::{Request, Method, Status, HeaderName};

#[derive(Serialize, Deserialize)]
pub struct PublicKeyStrings {
    pub q: String,
    pub p: String,
    pub y: String,
    pub g: String
}

#[derive(Serialize, Deserialize)]
pub struct PlaintextProof {
    challenge: String,
    commitment: String,
    response: String
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedChoice {
    alpha: String,
    beta: String
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedVote {
    choices: Vec<EncryptedChoice>,
    issue_date: String,
    proofs: Vec<PlaintextProof>
}

#[derive(Serialize, Deserialize)]
pub struct VoteRequest {
    vote: String, 
    vote_hash: String
}

fn get_public_key(client: &Client, get_election_url: &String) 
-> Result<PublicKey<BigintCtx::<P2048>>, Error>
{
    event!(
        Level::DEBUG,
        get_election_url = get_election_url,
    );
    let response = client.request(
        Request::builder(Method::GET, get_election_url.parse()?).build()
    )?;

    let status = response.status();
    event!(Level::INFO, request_response_status = status.to_string());
    
    let body = response.into_body().to_string()?;
    event!(Level::INFO, request_response_body = body);

    if ! status.is_successful() {
        return Err("invalid-status".into());
    }

    let body_value: Value = serde_json::from_str(&body)?;
    if !body_value.is_object()
        || !body_value.as_object().unwrap()["payload"].is_object()
        || !body_value.as_object().unwrap()["payload"].as_object().unwrap()["pks"].is_string()

    {
        return Err("invalid-election-body".into());
    }

    let public_key_string = body_value
        .as_object()
        .unwrap()["payload"]
        .as_object()
        .unwrap()["pks"]
        .to_string()
        .replace("\\", "");
    // strip the initial and last chars.. (they are unneeded doble quotes)
    let public_key_string = &public_key_string[1..public_key_string.len()-1];
    event!(Level::DEBUG, "public_key_string='{}'", public_key_string);

    let public_key_list: Vec<PublicKeyStrings> = 
        serde_json::from_str(&public_key_string)?;
    if public_key_list.len() != 1 {
        return Err("more-than-one-public-key".into());
    }
    let public_key_obj = &public_key_list[0];
    
    let context = BigintCtx::<P2048>::new();
    return Ok(
        PublicKey::from_element(
            &context.element_from_string_radix(&public_key_obj.y, 10)?,
            &context
        )
    );
}

fn get_voter_id(auth_token: &String) -> Result<String, Error> {
    let (_, signed_data) = auth_token.rsplit_once('/').ok_or("")?;
    let (voter_id, _) = signed_data.split_once(':').ok_or("")?;
    return Ok(voter_id.into());
}

pub fn get_hash(data: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hashed = hasher.finalize();
    return hex::encode(&hashed)
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<ConnectEvent>) -> Result<Value, Error> {

    let (connect_event, connect_context) = event.into_parts();
    event!(
        Level::DEBUG,
        connect_event = serde_json::to_string(&connect_event)?,
        connect_context = serde_json::to_string(&connect_context)?
    );

    // Example RECORD_VOTE_URL, where votes will be posted: 
    // https://clientname.example.com/elections/api/election/{{election_id}}/voter/{{voter_id}}
    // Note that:
    // - {{election_id}} will be substituted with the election id
    // - {{voter_id}} will be substituted with the voter id
    let record_vote_url_template = env::var("RECORD_VOTE_URL")?;
    event!(Level::INFO, record_vote_url_template);

    // Example GET_ELECTION_URL, used to fetch election config:
    // https://clientname.example.com/elections/api/election/{{election_id}}
    // Note that {{election_id}} will be substituted with the election id
    let get_election_url_template = env::var("GET_ELECTION_URL")?;
    event!(Level::INFO, get_election_url_template);

    let vote_encoding_array_str = env::var("VOTE_ENCODING_ARRAY")?;
    event!(Level::INFO, vote_encoding_array_str);

    let vote_text: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("Vote")
        .ok_or(String::from("Vote contact data attribute missing"))?;
    event!(Level::DEBUG, vote_text);

    let auth_token: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("AuthToken")
        .ok_or(String::from("AuthToken contact data attribute missing"))?;
    event!(Level::DEBUG, auth_token);

    let election_id: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("ElectionId")
        .ok_or(String::from("ElectionId contact data attribute missing"))?;
    event!(Level::DEBUG, election_id);

    let get_election_url = get_election_url_template
        .replace("{{election_id}}", election_id);

    let client = Client::new();
    let public_key = get_public_key(&client, &get_election_url)?;
    let vote_encoding_array: HashMap<String, u32> = 
        serde_json::from_str(&vote_encoding_array_str)?;

    let context = BigintCtx::<P2048>::new();
    
    let vote_int: &u32 = vote_encoding_array.get(vote_text).ok_or("")?;
    let vote_encoded = context.encode(&BigUint::from(*vote_int))?;

    let (cyphertext, plaintext_proof, debug_str) = public_key
        .encrypt_and_pok_old_version(
            &vote_encoded,
            &vec![]
        );
    event!(Level::DEBUG, old_version_debug = debug_str);

    let plaintext_proof_struct = PlaintextProof {
        challenge: plaintext_proof.challenge.to_string_radix(10),
        commitment: plaintext_proof.commitment.to_string_radix(10),
        response: plaintext_proof.response.to_string_radix(10)
    };
    let encrypted_ballot = EncryptedChoice {
        alpha: cyphertext.gr().to_string_radix(10),
        beta: cyphertext.mhr().to_string_radix(10)
    };
    let encrypted_ballot = EncryptedVote {
        choices: vec![
            encrypted_ballot
        ],
        issue_date: Utc::now().format("%Y/%m/%d").to_string(),
        proofs: vec![
            plaintext_proof_struct
        ]
    };
    let encrypted_ballot_str: String = serde_json::to_string(&encrypted_ballot)?;
    event!(Level::INFO, encrypted_ballot_str);

    let vote_hash = get_hash(&encrypted_ballot_str);
    event!(Level::INFO, vote_hash = vote_hash);

    let vote_request = VoteRequest {
        vote: encrypted_ballot_str,
        vote_hash: vote_hash.clone()
    };
    let vote_request_str: String = serde_json::to_string(&vote_request)?;
    event!(Level::INFO, vote_request_str);

    let voter_id = get_voter_id(auth_token)?;
    event!(Level::INFO, voter_id);

    let record_vote_url = record_vote_url_template
        .replace("{{election_id}}", election_id)
        .replace("{{voter_id}}", &voter_id);
    event!(Level::DEBUG, record_vote_url);

    event!(
        Level::DEBUG,
        record_vote_url = record_vote_url,
        request_authorization_header = auth_token,
        request_body = vote_request_str
    );
    let response = client.request(
        Request::builder(Method::POST, record_vote_url.parse()?)
            .with_header(HeaderName::AUTHORIZATION, auth_token.as_str())?
            .with_header(HeaderName::CONTENT_TYPE, "application/json")?
            .with_body(vote_request_str)
    )?;
    
    let status = response.status();
    event!(Level::INFO, request_response_status = status.to_string());
    
    let body = response.into_body().to_string()?;
    event!(Level::INFO, request_response_body = body);
    
    match status {
        Status::OK => {
            let vote_hash_ssml = String::from(&vote_hash[..8]).chars().fold(
                String::from(""),
                |hash_ssml, character| {
                    format!(
                        "{}<s><say-as interpret-as=\"verbatim\">{}</say-as></s>",
                        hash_ssml,
                        character
                    )
                }
            );
            let ret_value = json!({
                "VoteHashStartSSML": &vote_hash_ssml
            });
            event!(Level::DEBUG, ret_value = ret_value.to_string());

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
    event!(Level::INFO, "starting up `record_vote` lambda");

    run(service_fn(function_handler)).await
}
