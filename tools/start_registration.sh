#!/bin/bash

. .bash_config
curl -XPOST $SQUIRREL_URL/admin/registration/start 2>/dev/null
echo "Registration has started"
