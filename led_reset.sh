#!/usr/bin/env bash

. /root/gehma_server/.env && psql $DATABASE_URL -c "UPDATE users SET led = false;";
