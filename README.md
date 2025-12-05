# Overview

ETH Narwhal Node is a Rust-based, multi-threaded blockchain node built on a modern modular stack. It integrates REVM for Ethereum Virtual Machine execution, Narwhal–Bullshark–style asynchronous BFT consensus, libp2p for peer-to-peer networking, and RocksDB for persistent chain state. The node is post-quantum secure (using ML-DSA from the pqcrypto suite) and designed for cross-chain interoperability with Solana, Sui, and Aptos.

# Structure
├── Cargo.toml
└── src/
    ├── main.rs         # Multi-threaded runtime entrypoint
    ├── crypto.rs       # Post-quantum key & signature module
    ├── db.rs           # RocksDB-backed state & block store
    ├── evm.rs          # REVM-based EVM transaction executor
    ├── p2p.rs          # libp2p Gossip networking
    ├── consensus.rs    # Narwhal–Bullshark DAG consensus engine
    ├── rpc.rs          # Ethereum JSON-RPC server
    └── types.rs        # Common primitives and structs
