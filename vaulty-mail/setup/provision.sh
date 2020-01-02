#!/bin/bash

sudo apt-get install postfix

# Setup the virtual mail user
sudo mkdir /var/vmail
sudo groupadd -g 5000 vmail
sudo useradd -m -d /var/vmail -s /bin/false -u 5000 -g vmail vmail
sudo chown -R vmail:vmail /var/vmail

# Copy over Postfix configs
# NOTE: This includes filter binary!
cp -R $SETUP_DIR/postfix/* /etc/postfix/

# FIXME: Make this setup Postgres instead (?)
/usr/sbin/postmap /etc/postfix/vmail
