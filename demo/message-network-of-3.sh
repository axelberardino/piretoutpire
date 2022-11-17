#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "MESSSAGE BETWEEN 2 USERS THROUGH ANOTHER ONE"
echo

section "Clean folder $DATA_DIR"
launch "rm -rf $BASE_DIR/*"
launch "mkdir -p $BASE_DIR/a $BASE_DIR/b $BASE_DIR/c"

LOG="$BASE_DIR/log"

section "Launch peer A"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4001\" --peer-id=1 --dht-filename=$BASE_DIR/a.dht seed" "$LOG"
server_pid1=$!
sleep 1

section "Bootstrap peer B (from peer A)"
launch "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=$BASE_DIR/b.dht bootstrap 127.0.0.1:4001"

section "Launch peer B"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4002\" --peer-id=2 --dht-filename=$BASE_DIR/b.dht seed" "/dev/null"
server_pid2=$!
sleep 1

section "Bootstrap peer C (from peer B)"
launch "./$BIN --server-addr=\"127.0.0.1:4003\" --peer-id=3 --dht-filename=$BASE_DIR/c.dht bootstrap 127.0.0.1:4002"

section "Now, we have this network C -> B -> A, let's send a message from C to A"
launch "./$BIN --server-addr=\"127.0.0.1:4003\" --peer-id=3 --dht-filename=$BASE_DIR/c.dht message 1 \"Hello A, this is C speaking!\""

section "Peer A (seed server) should have received a ping from C and get its message"
launch "cat $LOG"

section "Stop the seed peer"
launch "kill $server_pid1 $server_pid2 &>/dev/null"

wait
