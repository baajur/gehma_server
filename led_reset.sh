#!/usr/bin/env bash

. /home/kper/.env && psql $DATABASE_URL -c "UPDATE users SET led = false WHERE led = true;";
