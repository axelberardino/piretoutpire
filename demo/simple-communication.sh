#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "SIMPLE COMMUNICATION BETWEEN 2 PEERS"
echo

section "Clean folder $DATA_DIR"
launch "rm -rf $BASE_DIR/*"
launch "mkdir -p $BASE_DIR/a $BASE_DIR/b"

LOG="$BASE_DIR/log"

section "Launch peer A"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4000\" --peer-id=1 --dht-filename=/tmp/p2p-demo/a.dht seed" "$LOG"
server_pid=$!
sleep 1

section "Launch peer B"
launch "./$BIN --peer-id=2 bootstrap 127.0.0.1:4000"
sleep 1

section "Peer A (seed server) should have received a ping from B, acknowledge B as 'peer 2' and sent back an empty closest peers"
launch "cat $LOG"

section "Stop the seed peer"
launch "kill $server_pid &>/dev/null"

wait
