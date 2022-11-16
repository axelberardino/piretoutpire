#!/bin/bash

BASE_DIR=/tmp/p2p-demo
DEMOS="simple-communication.sh message-network-of-2.sh message-network-of-3.sh"
# DEMOS="message-network-of-3.sh"

trap "echo" SIGTERM

echo -e "\033[31;1m"
echo "  _____  ______ __  __  ____  "
echo " |  __ \|  ____|  \/  |/ __ \ "
echo " | |  | | |__  | \  / | |  | |"
echo " | |  | |  __| | |\/| | |  | |"
echo " | |__| | |____| |  | | |__| |"
echo " |_____/|______|_|  |_|\____/ "
echo "                              "
echo -e "\033[0m"
echo
echo "CTRL + C will skip the current test, not the entire demo"
echo "Press CTRL + C twice to abort the demo"
echo
read -p "Press enter to start the demo"
echo

cur_dir=$(basename $(pwd) )
prefix=demo
if [ "$cur_dir" = "demo" ]; then
    prefix=.
fi

for demo in $DEMOS; do
    ./${prefix}/${demo} "$BASE_DIR"
    read -p "Press enter to start the next demo"
done

BIN="pire2pire"

if [ -z "$BIN" ]; then
    echo "can't found binary, check the make build command has been launch in the root repository"
fi
