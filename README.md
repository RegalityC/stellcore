# veridrop

A Soroban smart contract for Merkle-proof-based token distribution on the Stellar network. Recipients claim tokens by submitting a cryptographic proof - only the Merkle root is stored on-chain, keeping deployment costs independent of distribution size.

---

## Table of Contents

- [Background](#background)
- [How It Works](#how-it-works)
- [Contract API](#contract-api)
- [Error Reference](#error-reference)
- [Leaf Hashing and Proof Verification](#leaf-hashing-and-proof-verification)
- [Security Considerations](#security-considerations)
- [Development](#development)

---

## Background

Rather than storing every recipient's address and claimable amount on-chain, only the SHA-256 Merkle root of the full distribution list is persisted. Recipients receive their leaf data and proof off-chain. At claim time, the contract recomputes the root from the submitted proof and verifies it against the stored value. This pattern is trust-minimized, gas-efficient at scale, and resistant to unauthorized claims.

---

## How It Works

1. **Off-chain:** Build a Merkle tree from `(index, receiver, amount)` tuples. Distribute leaf data and proofs to recipients.
2. **Deployment:** Store the Merkle root and token address on-chain. Transfer the total token allocation into the contract from a funding source.
3. **Claim:** A recipient submits their tuple and sibling-hash proof. The contract recomputes the root, verifies it, marks the index as used, and transfers tokens.

---

## Contract API

### `__constructor`
```rust
fn __constructor(
    env: Env,
    root_hash: BytesN<32>,
    token: Address,
    funding_amount: i128,
    funding_source: Address,
)
```

Initializes the contract. Called once at deployment.

| Parameter | Type | Description |
|---|---|---|
| `root_hash` | `BytesN<32>` | SHA-256 Merkle root of the distribution tree |
| `token` | `Address` | Stellar Asset Contract address for the token to distribute |
| `funding_amount` | `i128` | Total tokens to move into this contract at initialization |
| `funding_source` | `Address` | Address that funds the contract; must have pre-approved the transfer |

Stores `root_hash` and `token` in persistent storage, then transfers `funding_amount` from `funding_source` to the contract.

---

### `claim`
```rust
fn claim(
    env: Env,
    index: u32,
    receiver: Address,
    amount: i128,
    proof: Vec<BytesN<32>>,
) -> Result
```

Verifies a proof and transfers tokens to the receiver.

| Parameter | Type | Description |
|---|---|---|
| `index` | `u32` | Leaf position in the distribution tree; used as the deduplication key |
| `receiver` | `Address` | Recipient address; must match the committed value |
| `amount` | `i128` | Token amount; must match the committed value |
| `proof` | `Vec<BytesN<32>>` | Ordered sibling hashes from leaf to root |

**Execution steps:**
1. Reject if `index` is already claimed -> `AlreadyClaimed`
2. Hash `(index, receiver, amount)` to produce the leaf
3. Recompute the root by iterating through `proof`
4. Reject if the recomputed root does not match -> `InvalidProof`
5. Mark `index` as claimed
6. Transfer `amount` tokens to `receiver`

---

## Error Reference

| Variant | Code | Description |
|---|---|---|
| `AlreadyClaimed` | `1` | This index has already been claimed. Each index is single-use. |
| `InvalidProof` | `2` | The recomputed root does not match the stored root. |

---

## Leaf Hashing and Proof Verification

**Leaf construction:**
```
leaf = SHA-256( index || receiver || amount )
```
Encoding order and endianness must match exactly between the off-chain tree builder and the contract.

**Root recomputation:**
```
current = leaf_hash
for each sibling in proof:
    if index % 2 == 0:
        current = SHA-256(current || sibling)
    else:
        current = SHA-256(sibling || current)
    index = index / 2

assert current == stored_root
```

Node ordering at each level is determined by index parity, consistent with the standard binary Merkle tree convention.

---

## Security Considerations

- **Root is immutable.** Verify the off-chain tree is correct before deploying - the root cannot be updated after initialization.
- **Funding authorization.** The `funding_source` must call `approve` on the token contract before deployment. The contract does not handle this.
- **Index uniqueness.** Deduplication is per-index. The off-chain tree must assign a unique index to every entry - duplicates are not detectable on-chain.
- **Proof ordering.** Sibling hashes must be submitted in the order produced by the tree builder. Any reordering produces a different root.
- **Amount precision.** `i128` amounts must be consistent between the distribution list and the token contract's decimal handling.

---

## Development

### Prerequisites

- Rust stable toolchain
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli)

### Build
```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/<contract>.wasm`

### Test
```bash
cargo test
```

| Test | Description |
|---|---|
| Valid claim | Correct proof and unclaimed index -> tokens transferred |
| Double claim | Already-claimed index -> `AlreadyClaimed` |
| Invalid proof | Tampered proof -> `InvalidProof` |

## Deployment

The deployment performed with this project is confirmed with the following:

| Field | Submission |
|-------|----------------|
| **GitHub Repository** | `https://github.com/RegalityC/veridrop` |
| **Contract ID** | CD5RUPOPD3CW5XZWPAMMC6HOY6EDOVSWWJ3IJB2DRDH4DULUDOAGDUUE |
| **Stellar Expert Link** | `https://stellar.expert/explorer/testnet/contract/CD5RUPOPD3CW5XZWPAMMC6HOY6EDOVSWWJ3IJB2DRDH4DULUDOAGDUUE` |

---

## Project Structure
```
.
├── src/
│   └── lib.rs       # Contract logic
├── Cargo.toml       # Dependencies
└── README.md
```
