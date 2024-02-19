#!/bin/bash
LOG=~/logs/server.log
touch $LOG
/home/frank/.local/bin/server --ip "0.0.0.0" --port 4545 --cert "/home/frank/certs/ripplein.space-dev.pem" --key "/home/frank/certs/ripplein.space-dev-key.pem" &>>$LOG &