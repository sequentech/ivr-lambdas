<!--
SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@nsequentech.io>

SPDX-License-Identifier: AGPL-3.0-only
-->
# ivr-lambdas

[![Chat][discord-badge]][discord-link]
[![Build Status][build-badge]][build-link]
[![codecov][codecov-badge]][codecov-link]
[![Dependency status][dependencies-badge]][dependencies-link]
[![License][license-badge]][license-link]
[![REUSE][reuse-badge]][reuse-link]

`ivr-lambdas` Contains the AWS Lambda functions of the Sequent Telephone Voting
System.

Note: This code is a MVP initially developed for a client that required telephone
voting. Currently it's not a generic system, and needs to be manually configured
for each election.

## Design

The IVR is just a wrapper around Sequent Voting Platform. It uses [AWS Connect]
to generate a simple IVR system for telephone voting, and requires the following
two AWS lambda functions with the Sequent backend API to work:
1. `authenticate_voter` lambda, that receives the credentials that the voter
provided telephonically and authenticates it, returning the bearer token
credential. This token will be stored in a [contact attribute] to be used later
when casting the vote.
2. `record_vote` lambda, that receives the intention of the vote that the voter
recorded using the telephone together with the bearer token credential obtained
from the previous call to the `authenticate_voter` lambda, and then encrypts
the vote and sends it to the ballot box.

## Development environment

ivr-lambdas uses [Github dev containers] to facilitate development. To start
developing ivr-lambdas, clone the github repo locally, and open the folder in
Visual Studio Code in a container. This will configure the same environment that
ivr-lambdas developers use, including installing required packages and VS Code
plugins.

We've tested this dev container for Linux x86_64 and Mac Os arch64
architectures. Unfortunately at the moment it doesn't work with Github
Codespaces as nix doesn't work on Github Codespaces yet. Also the current dev
container configuration for ivr-lambdas doesn't allow commiting to the git repo
from the dev container, you should use git on a local terminal.

## Building

ivr-lambdas uses the [Nix Package Manager] as its package builder. To build
ivr-lambdas, **first [install Nix]** correctly in your system (the dev container
already does this). If you're running the project on a dev container, you
shouldn't need to install it.

After you have installed Nix, enter the development environment with:

```bash
nix develop
```

You can build the lambdas using [cargo-lambda] as mentioned in the
[Rust runtime for AWS Lambda]:

```bash
cargo lambda build --release --arm64
```

## Testing lambdas locally

You can test the lambda functions locally. For example, you can run the
`authenticate_voter` lambda by executing in one terminal:

```bash
export LOGIN_URL=test_url
export USER_ID_KEY=user_id_key
export VOTER_PIN_KEY=voter_pin_key
cargo lambda watch -v --print-traces -- -C authenticate_voter
```

And then in another terminal:

```bash
cargo lambda invoke --data-file authenticate_voter/test/test_data_1.json
```

Note that:
1. Because of the limitations of the interaction of cargo-watch and cargo
workspaces, the lambda will be rebuild in the first invocation of the lambda.
2. This is just an example of how to invoke the lambda, but this example will
fail as it requires the proper configuration of the lambda environment
variables.

## Updating Cargo.toml

Use the following [cargo-edit] command to upgrade dependencies to latest
available version. This can be done within the `nix develop` environment:

```bash
cargo upgrade -Z preserve-precision
```

This repository doesn´t include a `Cargo.lock` file as it is intended to work as
a library. However for Wasm tests we keep a copy of the file on
`Cargo.lock.copy`. If you update Cargo.toml, keep the lock copy file in sync by
generating the lock file with `cargo generate-lockfile`, then `mv Cargo.lock
Cargo.lock.copy` and commit the changes.

[discord-badge]: https://img.shields.io/discord/1006401206782001273?style=plastic
[discord-link]: https://discord.gg/WfvSTmcdY8

[build-badge]: https://github.com/sequentech/ivr-lambdas/workflows/CI/badge.svg?branch=main&event=push
[build-link]: https://github.com/sequentech/ivr-lambdas/actions?query=workflow%3ACI

[codecov-badge]: https://codecov.io/gh/sequentech/ivr-lambdas/branch/main/graph/badge.svg?token=W5QNYDEJCX
[codecov-link]: https://codecov.io/gh/sequentech/ivr-lambdas

[dependencies-badge]: https://deps.rs/repo/github/sequentech/ivr-lambdas/status.svg
[dependencies-link]: https://deps.rs/repo/github/sequentech/ivr-lambdas

[license-badge]: https://img.shields.io/github/license/sequentech/ivr-lambdas?label=license
[license-link]: https://github.com/sequentech/ivr-lambdas/blob/master/LICENSE

[reuse-badge]: https://api.reuse.software/badge/github.com/sequentech/ivr-lambdas
[reuse-link]: https://api.reuse.software/info/github.com/sequentech/ivr-lambdas

[AWS Connect]: https://docs.aws.amazon.com/connect/?id=docs_gateway
[Github dev containers]: https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/introduction-to-dev-containers
[contact attribute]: https://docs.aws.amazon.com/connect/latest/adminguide/what-is-a-contact-attribute.html
[Nix Package Manager]: https://nixos.org/
[install Nix]: https://nixos.org/
[cargo-lambda]: https://www.cargo-lambda.info/
[Rust runtime for AWS Lambda]: https://github.com/awslabs/aws-lambda-rust-runtime#12-build-your-lambda-functions