# CLI Helper

```
PireToutPire
CLI for the p2p network

USAGE:
    pire2pire [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --connection-timeout <ms>
            Max wait time for initiating a connection (default is 200 ms)

        --dht-dump-frequency <ms>
            Frequency at which the dht is dump into the disk (default is 30 sec)

        --dht-filename <dht-filename>
            Config file for dht [default: /tmp/dht]

        --disable-recent-peers-cache
            Disable the recent peers cache. On small network, with non uniform id distribution,
            caching peers could be hard. The "recent" peers cache is used on top of the routing
            table, to help find peers. On big network, it's usually not needed and could be
            disactivated

    -h, --help
            Print help information

        --max-hop <nb>
            Max hop (empty = default behavior, search until not closer). Setting this option will
            enable a more greedy strategy for peers finding

        --peer-id <id>
            Peer id (empty = random)

        --read-timeout <ms>
            Max wait time for receiving a query (default is 200 ms)

        --server-addr <host:port>
            Listening address for receiving commands [default: 127.0.0.1:4000]

        --share-dir <share-dir>
            Where the downloaded files and the ones to seed are located [default: /tmp]

        --slowness <ms>
            Force this peer to wait X ms before answering each rpc (for debug purpose)

        --working-dir <working-dir>
            Where the downloaded files and the ones to seed are located [default: .]

        --write-timeout <ms>
            Max wait time for sending a query (default is 200 ms)

SUBCOMMANDS:
    announce            Send to closest node the crc of the file we're sharing
    bootstrap           Bootstrap this peer by giving a known peer address. Use it with a big
                            max-hop to discover peers on a small network
    direct-find-node    Directly ask the closest peers of a peer by its address
    download            Download a file, given its crc
    file-info           Ask a peer for file description, given its crc
    find-node           Find the given peer by its id, or return the 4 closest
    find-value          Find a value on the dht
    get-peers           Get the peers who are owning the wanted file
    help                Print this message or the help of the given subcommand(s)
    list                List all the known peers
    message             Send a message to a given peer
    ping                Ping a user from its peer id
    seed                Passively seed files and dht
    share-dir           Share a given file on the network
    share-file          Share a given file on the network
    store-value         Store a value on the dht
```
