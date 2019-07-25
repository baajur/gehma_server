#!/usr/bin/env bash

source .env && psql $DATABASE_URL -c "UPDATE users SET led = false;";
