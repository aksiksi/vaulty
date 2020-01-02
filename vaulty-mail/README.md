# vaulty-mail

Backend code that handles everything related to mail.  All tools in this directory are meant to run on the mail server directly.

## vaulty-filter

A Postfix filter that is triggered upon receipt of an email.  The filter is optimized for speed in order to minimize time spent in the Postfix queue.

As soon as the required checks are done (if any), the filter forwards the email info to `vaulty-mgr` and terminates.

## vaulty-mgr

A simple HTTP server that listens for incoming mail from `vaulty-filter`.  The bulk of the mail handling logic is done here.

In a nutshell:

1. Updates any counters or state in the DB.
2. Stores mail in Dropbox/GDrive/etc. based on config in DB.
3. TODO

## vaulty-setup

Setup scripts and tools for provisioning a `vaulty-mail` instance/server. This includes installing and configuring Postfix.
