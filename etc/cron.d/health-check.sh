#!/bin/env bash

# This is the cron entry:
# This runs every minute.
# 0 * * * * root /root/bin/unsaferust-health-check.sh

curl -s https://unsaferust.org > /dev/null 2>&1
exitCode=$(echo $?);

if [ "$exitCode" -gt "0" ]
then
    export SENDGRID_API_KEY=$(cat /root/SENDGRID_API_KEY.txt);
    curl --request POST \
    --url https://api.sendgrid.com/v3/mail/send \
    --header "Authorization: Bearer $SENDGRID_API_KEY" \
    --header 'Content-Type: application/json' \
    --data '{"personalizations": [{"to": [{"email": "vagelis.prokopiou@gmail.com"}]}],"from": {"email": "vagelis.prokopiou@protonmail.com"},"subject": "unsaferust.org error","content": [{"type": "text/plain", "value": "unsaferust.org is down"}]}';
fi