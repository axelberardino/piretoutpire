#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "DIRECT MESSSAGE BETWEEN 2 USERS"
echo

section "Clean folder $DATA_DIR"
launch "rm -rf $BASE_DIR/*"
launch "mkdir -p $BASE_DIR/a $BASE_DIR/b"

LOG="$BASE_DIR/log"

section "Launch peer A"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4001\" --peer-id=1 --dht-filename=/tmp/p2p-demo/a.dht seed" "$LOG"
server_pid=$!
sleep 1

section "Bootstrap peer B"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=/tmp/p2p-demo/b.dht bootstrap 127.0.0.1:4001"

section "Send message from B to A"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=/tmp/p2p-demo/b.dht message 1 \"Hello A, this is B speaking!\""

section "Peer A (seed server) should have received a ping from B and get its message"
launch "cat $LOG"

section "Stop the seed peer"
launch "kill $server_pid &>/dev/null"

wait
