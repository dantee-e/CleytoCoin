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

## About

**CleytoCoin** is a cryptocurrency built using the Rust programming language. This project aims to create a secure, fast, and decentralized cryptocurrency to facilitate peer-to-peer transactions.

## Features

- **Proof of Work (PoW)** consensus mechanism
- Secure peer-to-peer transactions
- Fast block generation time
- High scalability and low latency
- Rust-based with a focus on performance and safety

## Getting Started

Follow these instructions to get your local instance of **CleytoCoin** up and running.

### Prerequisites

Ensure you have the following dependencies installed on your machine:

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo**: Cargo is included with Rust, and will be automatically installed when you install Rust.

### Installation

Clone this repository to your local machine:

```bash
$ git clone https://github.com/dantee-e/CleytoCoin.git
$ cd CleytoCoin 
```
To build and install the project:
``` bash
$ cargo build --release
```
This will compile the project and generate an optimized binary in the `target/release` directory.

## Usage

### Starting the node
To start the cryptocurrency node, use the following command:
``` bash
cargo run --bin CleytoCoin
```
The node will start and connect to the network. You can start mining or send/receive transactions.

### Creating a wallet
To generate a new wallet, run the following:
``` bash
cargo run --bin CleytoCoin-wallet generate
```
This will generate a private key and address for your wallet.

### Sending a transaction
To send a transaction, use the following command:
``` bash
cargo run --bin CleytoCoin-wallet send --to <recipient_address> --amount <amount> --private-key <your_private_key>
```

### Mining
Start mining by running:
``` bash
cargo run --bin CleytoCoin-miner start --mining-key <your_private_key>
```

### Stopping the node
To stop the node, press `CTRL+C` or run the following:
``` bash
cargo run --bin CleytoCoin stop
```

## Testing
To run the tests for the project, use the following command:
``` bash
cargo test
```
This will run all unit tests, integration tests, and any other tests defined in this project.
If you wish to run with output run:
``` bash
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
