# RGB Node

![Build](https://github.com/LNP-BP/rgb-node/workflows/Build/badge.svg)
![Lints](https://github.com/LNP-BP/rgb-node/workflows/Lints/badge.svg)

[![crates.io](https://meritbadge.herokuapp.com/rgb_node)](https://crates.io/crates/rgb_node)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

This is source for daemon executables and library that can run RGB backend. For
wallet and exchange integration please check an interface to it and demo
projects from [RGB SDK](https://github.com/LNP-BP/RGB-SDK).

This repository contains RGB node source code and SDK for wallet & server-side
development.

The node may run as a set of daemons (even in different docker containers);
a multi-threaded single process or as a set of managed threads within a
wallet app.

For an easy test setup that allows to experience the node's capabilities, you
can check out the [demo](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/doc/demo-0.1/).

## Design

The node (as other nodes maitained by LNP/BP Standards Association and Pandora
Core company subsidiaries) consists of multiple microservices, communicating
with each other via LNP ZMQ RPC interface.

[Node architecture](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/doc/node_arch.jpeg)

The set of microservices representing node can run as either:
1) single daemon process on desktop or a server;
2) cloud of docker-based daemons, one per microservice, with instance
   scalability and in geo-distributed environment;
3) inside a single mobile app as threads;
4) and even different nodes can be combined in their services between themselves
   into a single executables/mobile apps;
5) all P2P communications are end-to-end encrypted and work over Tor.

Other nodes, designed an maintained by LNP/BP Standards Association with the
same architecture include:
* [LNP Node](https://github.com/LNP-BP/lnp-node) for running Lightning Network
  Protocol (LNP) and Generalized Lightning Channels (GLC).
* [BP Node](https://github.com/LNP-BP/bp-node) for indexing bitcoin blockchain
  (you may think of it as a more efficient Electrum server alternative)
* Bifrost – node for storing/passing client-side-validated data with watchtower
  functionality and used for Storm/RGB/DEX infrastructure

Other third parties provide their own nodes designed with the same architecture,
for instance:
* [Keyring](https://github.com/pandoracore/keyring) for managing private key
  accounts, storage and signatures with support for miniscript and PSBTs

## Project organization & architecture

* [`src/api/`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/api/) – 
  LNP messages for all daemons used for message bus
* [`src/bin/`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/bin/) – 
  binaries for daemons & CLI launching main process
* [`src/cli/`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/cli/) – 
  CLAP-based command line API talking to message bus
* [`src/i8n/`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/i8n/) – 
  functions exposed to FFI talking to message bus
* `src/<name>/` – service/daemon-specific code:
  - [`src/stash/`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/stash) – 
    daemon managing RGB stash data and its storage;
    you may  configure it (with either config file, environment vars or
    command-line arguments) to use different forms of storage drivers;
  - [`src/contracts`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/contracts) – 
    daemons supporting high-level APIs for
    working with different forms of RGB Schema: RGB-20 (fungible assets),
    RGB-21 (collectionables/NFTs) etc;
  - [`src/rgbd`](https://github.com/LNP-BP/rgb-node/tree/v0.2.0/src/rgbd) – 
    daemon orchestrating bootstrapping of stash and
    contracts daemons

Each daemon (more correctly "microservice", as it can run as a thread, not
necessary a process) or other binary (like CLI tool) follows the same
organization concept for module/file names:
* `error.rs` – daemon-specific error types;
* `config.rs` – CLAP arguments & daemon configuration data;
* `runtime.rs` – singleton managing main daemon thread and keeping all ZMQ/P2P
  connections and sockets; receiving and processing messages through them;
* `processor.rs` – business logic functions & internal state management which
  does not depend on external communications/RPC;
* `index/`, `storage/`, `cache/` – storage interfaces and engines;
* `db/` – SQL-specific schema and code, if needed.

## Install

Minimum supported rust compiler version (MSRV): 1.45

### Local

To compile the node, please install [cargo](https://doc.rust-lang.org/cargo/),
then run the following commands:

#### Linux

    sudo apt update
    sudo apt install -y build-essential cmake libpq-dev libsqlite3-dev libssl-dev libzmq3-dev pkg-config

#### MacOS

    brew install cmake libpq sqlite pkg-config openssl zmq

#### Both

    git clone https://github.com/LNP-BP/rgb-node.git
    cd rgb-node
    cargo install --all-features --bins --path .

Now, to run the node you can execute

    rgbd --data-dir ~/.rgb --bin-dir ~/.cargo/bin -vvvv --contract fungible

### In docker

In order to build and run a docker image of the node, run:
```bash
docker build -t rgb-node .
docker run --rm --name rgb_node rgb-node
```

## Using

First, you need to start daemons:
`rgbd -vvvv -d <data_dir> -b <bin_dir> --contract fungible`
where `bin_dir` is a directory with all daemons binaries (usually
`~/.cargo/bin` from repo source after `cargo install --all-features --bins --path .`
command).

Issuing token:
`rgb-cli -d <data_dir> -vvvv fungible issue TCKN "SomeToken" <supply>@<txid>:<vout>`

Next, list your tokens
`rgb-cli -d <data_dir> -vvvv fungible list`

Do an invoice
`rgb-cli -d <data_dir> -vvvv fungible invoice <contract_id> <amount> <txid>:<vout>`,
where `<contract_id>` is id of your token returned by the last call, and
`<txid>:<vout>` must be a transaction output you are controlling.

Save the value of the binding factor you will receive: it will be required in
the future to accept the transfer. Do not share it!
Send the invoice string to the payee.

Doing transfer: this requires preparation of PSBT; here we use ones from our
sample directory
`rgb-cli -d <data_dir> -vvvv fungible transfer "<invoice>" test/source_tx.psbt <consignment_file> test/dest_tx.psbt -i <input_utxo> [-a <amount>@<change_utxo>]`
NB: input amount must be equal to the sum of invoice amount and change amounts.

This will produce consignment. Send it to the receiving party.

The receiving party must do the following:
`rgb-cli -d <data_dir> -vvvv fungible accept <consignment_file> <utxo>:<vout> <blinding>`,
where `utxo` and the `blinding` must be values used in invoice generation

## Developer guidelines

In order to update the project dependencies, run `cargo update`.
If any dependency updates, the `Cargo.lock` file will be updated, keeping
track of the exact package version.

After an update, run tests (`cargo test`) and manually test the software
in order to stimulate function calls from updated libraries.

Please check [software using RGB Node](#software-using-rgb-node) for integration
examples.

If any problem arises, open an issue.


## Software using RGB Node

* [Bitcoin Pro](https://github.com/pandoracore/bitcoin-pro): tool for
  RGB asset issue & management for asset by the issuers
* [MyCitadel](https://github.com/mycitadel) Bitcoin, LN & RGB
  enabled wallet service with support for other LNP/BP protocols
