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
launch_bg "./$BIN --server-addr=\"127.0.0.1:4001\" --peer-id=1 --dht-filename=$BASE_DIR/a.dht seed" "$LOG"
server_pid=$!
sleep 1

section "Launch peer B"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=$BASE_DIR/b.dht bootstrap 127.0.0.1:4001"

section "Peer A (seed server) should have received a ping from B,\n\
    acknowledge B as 'peer 2' and\n\
    sent back a list peers with only 'peer 2' in it"
launch "cat $LOG"

section "Stop the seed peer"
launch "kill $server_pid &>/dev/null"

wait
