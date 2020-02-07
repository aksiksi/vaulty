#!/bin/bash
# Run from this directory
VAULTY_URL=$1

curl -d "@email.json" -X POST http://$VAULTY_URL:7777/postfix/email
curl -d "@attachment.json" -X POST http://$VAULTY_URL:7777/postfix/attachment
