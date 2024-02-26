#!/bin/bash
LOG=~/logs/client.log
touch $LOG
/home/frank/.local/bin/client --ip "192.168.1.110" --port 4545 --file $1 &>>$LOG
