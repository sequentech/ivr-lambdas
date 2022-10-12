#!/bin/bash
# SPDX-FileCopyrightText: 2022 Eduardo Robles <edu@sequentech.io>
#
# SPDX-License-Identifier: AGPL-3.0-only


cd /workspaces/ivr-lambdas
export REGION=ca-central-1
export AWS_SHARED_CREDENTIALS_FILE=/workspaces/ivr-lambdas/deployment/aws-credentials
export AUTHENTICATE_VOTER_ENV_FILE=/workspaces/ivr-lambdas/deployment/authenticate_voter.env_vars
export RECORD_VOTE_ENV_FILE=/workspaces/ivr-lambdas/deployment/record_vote.env_vars
export IAM_ROLE=arn:aws:iam::581718213778:role/ivr-lambda-role

cargo lambda deploy \
    --verbose \
    --region $REGION \
    --env-file $AUTHENTICATE_VOTER_ENV_FILE \
    --iam-role $IAM_ROLE \
    authenticate_voter

cargo lambda deploy \
    --verbose \
    --region $REGION \
    --env-file $AUTHENTICATE_VOTER_ENV_FILE \
    --iam-role $IAM_ROLE \
    record_vote
