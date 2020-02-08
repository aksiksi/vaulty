#!/bin/bash
# Run from this directory
VAULTY_URL=$1

curl -d "@email.json" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer TEST123" http://$VAULTY_URL:7777/postfix/email

# Not JSON anymore!
#curl -d "@attachment.json" -H "Content-Type: application/json" -X POST -H "Authorization: Bearer TEST123" http://$VAULTY_URL:7777/postfix/attachment
