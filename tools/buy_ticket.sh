#!/bin/bash

. .bash_config

curl -XPOST $REGISTRATION_URL/buy_ticket?_access_token=Test 2>/dev/null
