#!/bin/bash

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi
source "${prefix}/utils.sh" &>/dev/null

title "FILE SHARING IN A BIG NETWORK"
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

section "Let's bootstrap 20 peers, as a chain"
previous=1
for i in $(seq 2 20); do
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

mkdir -p "$BASE_DIR/21/files"
curl "http://0217021.free.fr/portfolio/axel.berardino/img/zenly-versions.png" > "$BASE_DIR/21/files/test.png" 2>/dev/null
mkdir -p "$BASE_DIR/22/files"
cp "$BASE_DIR/21/files/test.png" "$BASE_DIR/22/files/test.png"
mkdir -p "$BASE_DIR/23/files"
cp "$BASE_DIR/21/files/test.png" "$BASE_DIR/23/files/test.png"
mkdir -p "$BASE_DIR/24/files"
cp "$BASE_DIR/21/files/test.png" "$BASE_DIR/24/files/test.png"

section "Bootstrap peers 21, 22, 23, 24 (from peer 20)"
launch "./$BIN --server-addr=\"127.0.0.1:4021\" --peer-id=21 --dht-filename=$BASE_DIR/21/dht bootstrap 127.0.0.1:4020"
section "Bootstrap a new peer 22 (from peer 20)"
launch "./$BIN --server-addr=\"127.0.0.1:4022\" --peer-id=22 --dht-filename=$BASE_DIR/22/dht bootstrap 127.0.0.1:4020"
section "Bootstrap a new peer 23 (from peer 20)"
launch "./$BIN --server-addr=\"127.0.0.1:4023\" --peer-id=23 --dht-filename=$BASE_DIR/23/dht bootstrap 127.0.0.1:4020"
section "Bootstrap a new peer 24 (from peer 20)"
launch "./$BIN --server-addr=\"127.0.0.1:4024\" --peer-id=24 --dht-filename=$BASE_DIR/24/dht bootstrap 127.0.0.1:4020"

LOG1="$BASE_DIR/log01"
LOG2="$BASE_DIR/log02"
LOG3="$BASE_DIR/log03"
LOG4="$BASE_DIR/log04"

section "These 4 peers will now serve a copy of the file"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4021\" --peer-id=21 --dht-filename=$BASE_DIR/21/dht --working-dir=$BASE_DIR/21/files --share-dir=$BASE_DIR/21/files share-file \"$BASE_DIR/21/files/test.png\"" "$LOG1"
server_pids="$server_pids $!"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4022\" --peer-id=22 --dht-filename=$BASE_DIR/22/dht --working-dir=$BASE_DIR/22/files --share-dir=$BASE_DIR/22/files share-file \"$BASE_DIR/22/files/test.png\"" "$LOG2"
server_pids="$server_pids $!"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4023\" --peer-id=23 --dht-filename=$BASE_DIR/23/dht --working-dir=$BASE_DIR/23/files --share-dir=$BASE_DIR/23/files share-file \"$BASE_DIR/23/files/test.png\"" "$LOG3"
server_pids="$server_pids $!"
launch_bg "./$BIN --server-addr=\"127.0.0.1:4024\" --peer-id=24 --dht-filename=$BASE_DIR/24/dht --working-dir=$BASE_DIR/24/files --share-dir=$BASE_DIR/24/files share-file \"$BASE_DIR/24/files/test.png\"" "$LOG4"
server_pids="$server_pids $!"
sleep 1

section "Peer 21, 22, 23 & 24 got the file"
launch "\ls -l $BASE_DIR/21/files/"
launch "\ls -l $BASE_DIR/22/files/"
launch "\ls -l $BASE_DIR/23/files/"
launch "\ls -l $BASE_DIR/24/files/"

mkdir -p "$BASE_DIR/25/files"

section "Bootstrap the peer receiver 25 (from peer 20)"
launch "./$BIN --server-addr=\"127.0.0.1:4025\" --peer-id=25 --dht-filename=$BASE_DIR/25/dht bootstrap 127.0.0.1:4020"

section "Now, we have a 24 peers network, let's try to download a file from a new peer"
launch "./$BIN --server-addr=\"127.0.0.1:4025\" --peer-id=25 --dht-filename=$BASE_DIR/25/dht --working-dir=$BASE_DIR/25/files --share-dir=$BASE_DIR/25/files --max-hop=99 download 1458724153"

section "View of the seeders"
launch "cat $LOG1"
launch "cat $LOG2"
launch "cat $LOG3"
launch "cat $LOG4"

section "^^In the log above ^^, we can see the file being shared by 4 peers giving them different chunks concurrently"

section "Peer 25 got the file as well!"
launch "\ls -l $BASE_DIR/25/files/"

section "Stop the seed peers"
launch "kill $server_pids &>/dev/null"

# wait
