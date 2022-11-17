#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "FILE SHARING BETWEEN 2 PEERS"
echo

section "Clean folder $DATA_DIR"
launch "rm -rf $BASE_DIR/*"
launch "mkdir -p $BASE_DIR/a $BASE_DIR/b"

LOG="$BASE_DIR/log"

curl "http://0217021.free.fr/portfolio/axel.berardino/img/zenly-versions.png" > "$BASE_DIR/a/test.png" 2>/dev/null

section "Let peer A share a file and seed it"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4001\" --peer-id=1 --dht-filename=$BASE_DIR/a/dht --working-dir=$BASE_DIR/a --share-dir=$BASE_DIR/a share-file \"$BASE_DIR/a/test.png\" " "$LOG"
server_pid=$!
sleep 1

section "A is sharing this file"
launch "\ls -l $BASE_DIR/a/test.png"

section "Bootstrap peer B"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=$BASE_DIR/b/dht --working-dir=$BASE_DIR/b/ --share-dir=$BASE_DIR/b bootstrap 127.0.0.1:4001"

section "B ask for a file by it's crc, and get it"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=$BASE_DIR/b/dht --working-dir=$BASE_DIR/b/ --share-dir=$BASE_DIR/b download 1458724153"

section "Peer A (seed server) should have received a download request from B and share the file by chunks"
launch "cat $LOG"

section "B now also own a copy of the file!"
launch "\ls -l $BASE_DIR/b/test.png"

section "Stop the seed peer"
launch "kill $server_pid &>/dev/null"

wait
