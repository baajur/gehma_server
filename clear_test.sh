#!/usr/bin/env bash

. ./.env && psql $DATABASE_URL -c "DELETE FROM users WHERE tele_num = '+43123456789'";
