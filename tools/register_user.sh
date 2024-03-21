#!/bin/bash

. .bash_config

USER_ID=$(curl -q -XPOST $SQUIRREL_URL/register 2>/dev/null | jq -r '.id')
echo "User Account ID: $USER_ID"
curl $SQUIRREL_URL/events/$USER_ID
