#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "STORING AND GETTING A VALUE IN A BIG NETWORK"
echo

section "Clean folder $DATA_DIR"
launch "rm -rf $BASE_DIR/*"

LOG="$BASE_DIR/log01"

mkdir -p "$BASE_DIR/01/files"
server_pids=""

section "Let's start peer 1"
launch_bg "./$BIN --server-addr=127.0.0.1:4001 --peer-id=1 --dht-filename=$BASE_DIR/01/dht --working-dir=$BASE_DIR/01/files --share-dir=$BASE_DIR/01/files seed" "$LOG"
server_pid1=$!
sleep 0.3

section "Let's bootstrap 30 peers, as a chain"
previous=1
for i in $(seq 2 30); do
    echo "Launching $i..."
    pi=$i
    while [[ ${#pi} -lt 2 ]] ; do
        pi="0${i}"
    done
    pp=$previous
    while [[ ${#pp} -lt 2 ]] ; do
        pp="0${previous}"
    done

    mkdir -p "$BASE_DIR/$pi/files"

    ./$BIN  --server-addr="127.0.0.1:40${pi}" --peer-id=${i} --dht-filename=$BASE_DIR/${pi}/dht --working-dir=$BASE_DIR/${pi}/files --share-dir=$BASE_DIR/${pi}/files bootstrap "127.0.0.1:40${pp}" #>/dev/null
    ./$BIN --server-addr="127.0.0.1:40${pi}" --peer-id=${i} --dht-filename=$BASE_DIR/${pi}/dht --working-dir=$BASE_DIR/${pi}/files --share-dir=$BASE_DIR/${pi}/files seed >$BASE_DIR/log${pi} &
    server_pids="$server_pids $!"
    sleep 0.3

    previous=$(echo "$previous + 1" | bc)
done

mkdir -p "$BASE_DIR/31/files"

section "Bootstrap a new peer 31 (from peer 30)"
launch "./$BIN --server-addr=\"127.0.0.1:4031\" --peer-id=31 --dht-filename=$BASE_DIR/31/dht bootstrap 127.0.0.1:4030"

section "Now, we have this network 31 -> 30 -> ... -> 1, let's try to get the message (not there yet)"
launch "./$BIN --server-addr=\"127.0.0.1:4031\" --peer-id=31 --dht-filename=$BASE_DIR/31/dht --max-hop=99 find-value 42"

section "Stop seeding for server 1"
launch "kill $server_pid1 &>/dev/null"

section "Let server 1 store a value in the network"
launch "./$BIN --server-addr=127.0.0.1:4001 --peer-id=1 --dht-filename=$BASE_DIR/01/dht --working-dir=$BASE_DIR/01/files --share-dir=$BASE_DIR/01/files --max-hop=99 store-value 42 \"answer to everything\""

section "Put server 1 in seed mode again"
launch_bg "./$BIN --server-addr=127.0.0.1:4001 --peer-id=1 --dht-filename=$BASE_DIR/01/dht --working-dir=$BASE_DIR/01/files --share-dir=$BASE_DIR/01/files seed" "$LOG"
sleep 0.5

section "Let's search again, this time the value has been found!"
launch "./$BIN --server-addr=\"127.0.0.1:4031\" --peer-id=31 --dht-filename=$BASE_DIR/31/dht --max-hop=99 find-value 42"

section "Stop the seed peers"
launch "kill $server_pids &>/dev/null"

wait
