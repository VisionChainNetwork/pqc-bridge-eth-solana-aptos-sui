# Overview

ETH Narwhal Node is a Rust-based, multi-threaded blockchain node built on a modern modular stack. It integrates REVM for Ethereum Virtual Machine execution, Narwhal–Bullshark–style asynchronous BFT consensus, libp2p for peer-to-peer networking, and RocksDB for persistent chain state. The node is post-quantum secure (using ML-DSA from the pqcrypto suite) and designed for cross-chain interoperability with Solana, Sui, and Aptos.

# Key Features
- Multi-threaded Runtime – Powered by tokio (8 workers by default)
- Asynchronous BFT Consensus – Narwhal–Bullshark DAG pipeline
- REVM Execution Engine – Fully EVM-compatible transaction state transitions
- libp2p Networking – Gossip propagation and peer synchronization
- RocksDB Persistence – Efficient local block/state storage
- Post-Quantum Cryptography – ML-DSA digital signatures (pqcrypto-mldsa)
- JSON-RPC API – Compatible with standard Ethereum tools (web3.js, ethers.js)
- Cross-Chain Hooks – Bridge scaffolds for Solana, Sui, and Aptos integration
