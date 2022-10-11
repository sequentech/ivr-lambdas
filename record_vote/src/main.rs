// SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
//
// SPDX-License-Identifier: AGPL-3.0-only

extern crate chrono;
use aws_lambda_events::event::connect::ConnectEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use reqwest;
use std::collections::HashMap;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use num_bigint::BigUint;
use strand::context::Ctx;
use strand::backend::num_bigint::{BigintCtx, P2048};
use strand::elgamal::PublicKey;
use chrono::prelude::*;
use tracing::{info, debug};

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

fn parse_public_key(public_key_string: &String) -> Result<PublicKey<BigintCtx::<P2048>>, Error> {
    let public_key_strings: PublicKeyStrings = 
        serde_json::from_str(public_key_string)?;
    
    let context = BigintCtx::<P2048>::new();
    return Ok(
        PublicKey::from_element(
            &context.element_from_string_radix(&public_key_strings.y, 10)?,
            &context
        )
    );
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<ConnectEvent>) -> Result<Value, Error> {

    let record_vote_url = env::var("RECORD_VOTE_URL")?;
    info!("record_vote_url = {}", record_vote_url);

    let public_key_str = env::var("ELECTION_PUBLIC_KEY")?;
    info!("public_key_str = {}", public_key_str);
    
    let vote_encoding_array_str = env::var("VOTE_ENCODING_ARRAY")?;
    info!("vote_encoding_array = {}", vote_encoding_array_str);
    
    let public_key = parse_public_key(&public_key_str)?;
    let vote_encoding_array: HashMap<String, u32> = 
        serde_json::from_str(&vote_encoding_array_str)?;

    let (connect_event, _) = event.into_parts();
    let context = BigintCtx::<P2048>::new();

    let vote_text: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("Vote")
        .unwrap();
    debug!("vote_text = {}", vote_text);
    
    
    let vote_int: &u32 = vote_encoding_array.get(vote_text).ok_or("")?;
    let vote_encoded = context.encode(&BigUint::from(*vote_int))?;
    let auth_token: &String = connect_event
        .details
        .contact_data
        .attributes
        .get("AuthToken")
        .unwrap();
    debug!("auth_token = {}", auth_token);

    let (cyphertext, plaintext_proof) = public_key.encrypt_and_pok(
        &vote_encoded,
        &vec![]
    );
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
        issue_date: Utc::today().format("%Y/%m/%d").to_string(),
        proofs: vec![
            plaintext_proof_struct
        ]
    };
    info!("encrypted_ballot = {}", serde_json::to_string(&encrypted_ballot)?);

    let client = reqwest::Client::new();
    let response = client.post(record_vote_url)
            .header("Authorization", auth_token)
            .json(&encrypted_ballot)
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

    info!("starting up record_vote lambda");

    run(service_fn(function_handler)).await
}
