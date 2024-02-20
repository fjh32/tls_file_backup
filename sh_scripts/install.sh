#!/bin/bash
LOCAL_PATH=~/.local/bin/
DIR=`dirname $0`
cd $DIR
PWD=`pwd`
cp file_backup_client.sh $LOCAL_PATH
cp file_backup_server.sh $LOCAL_PATH

cd $PWD/..
cp target/release/client $LOCAL_PATH
cp target/release/server $LOCAL_PATH