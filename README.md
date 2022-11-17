# PireToutPire

![logo](doc/pire_to_pire.png)

This project implements a simplified p2p network. It allows to send message from
a peer to another in a decentralized manner. Also, file sharing (like
bittorrent) is also handled.

More information is available in [doc section](doc/README.md).

# Running and playing with it

Launch a server like that:
```sh
cargo run -- seed
```

Then, there is a CLI (using clap and colored) which can be called like that:
```sh
cargo run -- help
```

It's also possible to generate the binary using:
```sh
make build
./demo/pire2pire --help
```

# Demo

A demo is provided to show all available features.

```sh
make demo
```

# Tests

Everything is unit tested. You can launch all the tests that with:
```sh
cargo test
```

# Linting

You'll find some custom linters made with dylint. There are some very specific
rules we were enforcing in my previous company. These are examples of what can
be done.

Launch clippy:
```sh
cargo clippy
```

Launch personal linter:
```sh
# You might need to install dylint
cargo install cargo-dylint dylint-link

# Launch it!
cargo dylint --all -- --all-features --bins --examples --tests
```

## VsCode integration

It's possible to change the default cargo command in VsCode to automatically
launch clippy and dylint. Very handy to view at glance any warnings.

In settings.json, just override the default command like that:
```json
    "rust-analyzer.checkOnSave.overrideCommand": [
        "cargo",
        "dylint",
        "--all",
        "--workspace",
        "--",
        "--all-targets",
        "--message-format=json"
    ]
```