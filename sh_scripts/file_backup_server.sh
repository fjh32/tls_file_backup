#!/bin/bash
LOG=~/logs/server.log
touch $LOG
/home/frank/.local/bin/server --cert "/home/frank/certs/ripplein.space-dev.pem" --key "/home/frank/certs/ripplein.space-dev-key.pem" --backup-dir "/drives/breen/backups/" &>>$LOG