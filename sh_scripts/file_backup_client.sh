#!/bin/bash
LOG=~/logs/client.log
touch $LOG
/home/frank/.local/bin/client --host "192.168.1.110" --file $1 &>>$LOG
