#!/usr/bin/env bash

. /home/web/gehma_server/.env && psql $DATABASE_URL -c "UPDATE users SET led = false WHERE led = true;";
