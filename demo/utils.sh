#!/bin/bash

finish()
{
    trap - SIGTERM # Disable sigterm trap to avoid signal recursion
    kill 0
}
trap "exit 0" SIGTERM
trap finish 0 1 2 3 13 # EXIT HUP INT QUIT PIPE

title()
{
    echo -e "\n\033[32;4m-=-=-=-=-=-=-=- ${1} -=-=-=-=-=-=-=-\033[0m\n"
}

section()
{
    echo -e "\n\033[34;1m--> ${1}\033[0m"
}

text()
{
    echo -e "\033[34;1m${1}\033[0m"
}

warn()
{
    echo -e "\033[31m/!\\ ${1} /!\\ \033[0m"
}

abort()
{
    echo "$1"
    exit 1
}

launch()
{
    echo -ne "\033[33m"
    set -f
    echo -n $1
    set +f
    echo -e "\033[0m"
    if [ $# -eq 1 ]; then
        eval $1
    else
        echo -e "\033[33mLog can be viewed here: $2\033[0m"
        eval $1 &> "$2"
    fi
}

launch_bg()
{
    echo -ne "\033[33m"
    set -f
    echo -n $1
    set +f
    echo -e "\033[0m"
    if [ $# -eq 1 ]; then
        eval $1 &
    else
        echo -e "\033[33mLog can be viewed here: $2\033[0m"
        eval $1 &> "$2" &
    fi
}

BIN=pire2pire
stat $SERVER &>/dev/null || SERVER=../$SERVER
stat $SERVER &>/dev/null || abort "Can't find pire2pire binary"

BASE_DIR="${1:-/tmp/p2p-demo}"
