# CleytoCoin

## Table of Contents

- [About](#about)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [Testing](#testing)
- [Contributing](#contributing)

# CleytoCoin is still under development, so some of the features listed underneath aren't yet functional. Those are marked with [Under development]

We appreciate the interest and are working towards making CleytoCoin a functional and reliable cryptocurrency, but currently it's still in it's early development stages.
Feel free to give suggestions in the meantime of what you'd like to see implemented in our project!

## About

**CleytoCoin** is a cryptocurrency built using the Rust programming language. This project aims to create a decentralized cryptocurrency to facilitate peer-to-peer transactions.

## Getting Started

Follow these instructions to get your local instance of a **CleytoCoin** node up and running.

### Prerequisites

Ensure you have the following dependencies installed on your machine:

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo**: Cargo is included with Rust, and will be automatically installed when you install Rust.

### Installation

Clone this repository to your local machine:

```bash
git clone https://github.com/dantee-e/CleytoCoin.git
cd CleytoCoin
```

To build and install the project:

```bash
cargo build --release
```

This will compile the project and generate an optimized binary in the `target/release` directory.

# How to use the binaries

Thanks to the [StructOpt](https://crates.io/crates/structopt) crate, whenever you feel in doubt about one of the features of **CleytoCoin**, you can use the flag --help of the CLI tool to see flags and arguments for said feature.

## Node Usage

### Starting the node

To start the cryptocurrency node, use the following command:

```bash
cargo run --bin node start
```

With the option of running the GUI:

```bash
cargo run --bin node start --gui
```

The server with the GUI will block the terminal, while just running the start creates a new process, which has to be killed afterwards using the [kill command](#killing-the-node)


The node will start and connect to the network. For now, only full nodes are available and they don't have yet the capacity for mining

### Killing the node

To kill the node, we follow the same pattern as before:

```bash
cargo run --bin node kill
```

## Wallet Usage

The `cleyto-coin-wallet` CLI has two main commands: `generate` (to create a wallet) and `send` (to send transactions).

### Generating a wallet
Generates a new keypair (private and public keys) for your wallet.

```bash
cargo run --bin cleyto-coin-wallet generate \
    --private-key-file <private-key-file> \
    --public-key-file <public-key-file> \
    [-p <password>]
vbnet
```
---

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--private-key-file` | Path where the generated private key will be stored | `./private.pem` |
| `--public-key-file`  | Path where the generated public key will be stored  | `./public.pem` |
| `-p, --password`     | Optional password to encrypt your private key | none |




### Sending a transaction

To send a transaction, you can use the same binary, but with the `send` subcommand

```bash
cargo run --bin cleyto-coin-wallet send \
    --recipient-key <recipient_public_key> \
    --sender-key <your_private_key> \
    --amount <amount> \
    [-p <password>]
```
Or, using key files:
```bash
cargo run --bin cleyto-coin-wallet send \
    --recipient-key-file <recipient_key_file> \
    --sender-key-file <your_private_key_file> \
    --amount <amount> \
    [-p <password>]
```

### Mining [Under develpment]

Start mining by running:

```bash
cargo run --bin cleyto-coin-miner start --mining-key <your_private_key>
```

### Stopping the node

To stop the node, on the terminal window running your server, press `CTRL+C`, `q` or `Esc`

**Running the server without the GUI is not yet implemented**
If you ran it without the GUI, use the command

```bash
cargo run --bin cleyto-coin stop # Not yet implemented
```

## Testing

To run the tests for the project, use the following command:

```bash
cargo test
```

This will run all unit tests, integration tests, and any other tests defined in this project.
If you wish to run with output run:

```bash
cargo test -- --nocapture
```

## Contributing

We welcome contributions to the CleytoCoin project. If you have an idea or find a bug, please feel free to submit an issue or a pull request.

1. Fork the repository
2. Create a new branch (`git checkout -b feature/feature-name`)
3. Make your changes
4. Commit your changes (`git commit -m 'Add feature'`)
5. Push to your forked repository (`git push origin feature/feature-name`)
6. Open a Pull Request
