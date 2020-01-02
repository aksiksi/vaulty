# Vaulty

Vaulty is service that allows users to send email directly into an online storage account (e.g., Dropbox).

## Overview

The service consists of two main components:

* `vaulty-mail`: handles setting up the mail server and forwarding received mail to the correct storage account.
* `vaulty-web`: web frontend and backend that the user interacts with.

These two services are "connected" through a Postgres database.  `vaulty-mail` uses the DB to figure out where to store a given email as well as to update usage information. `vaulty-web` provides a user signup flow and handles payments.

## TODO

- [ ] Provision scripts for `vaulty-mail`, `vaulty-web`, and DB
- [ ] Basic Django (?) web app
- [ ] IFTTT integration so users can "do stuff" upon receipt of an email
