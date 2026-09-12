# Migration Guide: AuraFlow Contracts — Arbitrum/Solidity → Stellar/Soroban

## Overview

This guide migrates the AuraFlow payroll smart contracts from Solidity (Foundry, Arbitrum) to Soroban (Rust, Stellar) with Lucent's ConfidentialToken for private payroll amounts.

**Current state:** Solidity contracts on Arbitrum Sepolia using Sablier streaming, OpenZeppelin, and Fhenix FHE stubs.
**Target state:** Soroban contracts on Stellar testnet with ConfidentialToken (Pedersen commitments on Grumpkin curve), PayrollVault, UltraHonk verifier, and Grumpkin auditor.

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | ≥ 1.83 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| `stellar` CLI | ≥ 25.2 | `cargo install --locked stellar-cli --features opt` |
| Node.js | ≥ 20 | `nvm install 20` |
| pnpm | ≥ 10 | `npm install -g pnpm` |

Verify installations:
```bash
rustc --version
stellar --version
node --version
```

---

## Step 1: Remove Old Solidity Project

```bash
cd /Users/user/Downloads/Contract/auraflow-contracts

# Remove Solidity source, tests, scripts
rm -rf src/ test/ script/ lib/ out/ cache/

# Remove Foundry config
rm foundry.toml foundry.lock .gitmodules

# Remove old package.json
rm package.json
```

Keep `.env.example`, `.gitignore`, `README.md` (update them later).

---

## Step 2: Initialize Soroban Project

```bash
cd /Users/user/Downloads/Contract/auraflow-contracts

# Initialize a new Soroban project
stellar init --name auraflow-contracts --overwrite
```

This creates:
```
auraflow-contracts/
├── Cargo.toml           (workspace)
├── contracts/
│   └── auraflow-contracts/   (default contract)
├── .env.example
├── .gitignore
└── README.md
```

---

## Step 3: Configure Cargo Workspace

Replace the generated `Cargo.toml` at the project root:

```toml
[workspace]
resolver = "2"
members = [
  "contracts/confidential-token",
  "contracts/auraflow-payroll",
  "contracts/verifier",
  "contracts/auditor",
]

[workspace.dependencies]
soroban-sdk = "21"
soroban-token-sdk = "21"
```

---

## Step 4: Implement Contracts

### 4a. Confidential Token Contract

**Path:** `contracts/confidential-token/`

**Purpose:** Wraps USDC into a confidential token using Pedersen commitments on the Grumpkin curve. Salary amounts are encrypted; addresses remain public.

**Reference:** Lucent's `contracts/token/` at https://github.com/ToluLabs/Lucent

**Key functions:**
- `initialize(admin, underlying_token, verifier)` — Set up the confidential token with USDC as underlying
- `register(user)` — Register a user's Grumpkin public key for confidential transfers
- `deposit(user, amount)` — Wrap USDC → confidential token (amount becomes a Pedersen commitment)
- `confidential_transfer(from, to, commitment, proof)` — Transfer using ZK proof that the commitment is valid
- `withdraw(user, amount, proof)` — Unwrap confidential token → USDC
- `get_commitment(user)` — Get the user's current Pedersen commitment (encrypted balance)

**Dependencies (Cargo.toml):**
```toml
[package]
name = "confidential-token"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "21"
```

**Note:** For a production implementation, use OpenZeppelin's `stellar-contracts` confidential token from the `feat/confidential-verifier-ultrahonk` branch as a git dependency. For the initial migration, implement a simplified version that stores Pedersen commitments and verifies UltraHonk proofs.

### 4b. AuraFlow Payroll Contract

**Path:** `contracts/auraflow-payroll/`

**Purpose:** Core payroll vault — fund treasury, process batch confidential transfers to employees, track employee vault registration status.

**Reference:** Lucent's `contracts/payroll/` (PayrollVault)

**Key functions:**
- `initialize(admin, confidential_token, usdc_contract)` — Set up payroll with confidential token reference
- `fund_treasury(employer, amount)` — Employer deposits USDC into the payroll vault
- `register_employee_vault(employee)` — Register employee's confidential token vault (calls `confidential-token.register()`)
- `process_payroll_batch(employer, recipients, commitments, proofs)` — Batch confidential transfer to multiple employees using ZK proofs
- `get_treasury_balance(employer)` — Read employer's on-chain treasury balance
- `withdraw_treasury(employer, amount)` — Employer withdraws from treasury

**Storage model:**
```
treasury_balances: Map<Address, i128>        // employer → USDC balance
employee_vaults: Map<Address, bool>          // employee → vault registered?
employee_employers: Map<Address, Address>    // employee → employer
```

### 4c. Verifier Contract

**Path:** `contracts/verifier/`

**Purpose:** UltraHonk verification key registry. Stores verification keys for the ZK proofs used by the confidential token.

**Reference:** Lucent's `contracts/verifier/`

**Key functions:**
- `initialize(admin)` — Set up verifier registry
- `register_vk(vk_hash, vk_data)` — Register a verification key
- `verify(proof, public_inputs, vk_hash)` — Verify a ZK proof against a registered verification key

**Note:** Use Nethermind's `rs-soroban-ultrahonk` as the verification backend. For the initial migration, this can be a stub that accepts pre-verified proofs.

### 4d. Auditor Contract

**Path:** `contracts/auditor/`

**Purpose:** Grumpkin auditor key registry. Auditors can decrypt transfer amounts for compliance purposes using viewing keys.

**Reference:** Lucent's `contracts/auditor/`

**Key functions:**
- `initialize(admin)` — Set up auditor registry
- `register_auditor(auditor_id, grumpkin_public_key)` — Register an auditor's Grumpkin public key
- `get_auditor_key(auditor_id)` — Retrieve an auditor's public key
- `decrypt_amount(encrypted_amount, auditor_key)` — Decrypt a Pedersen commitment (off-chain, auditor-side)

---

## Step 5: Deploy to Stellar Testnet

### 5a. Create a Stellar Testnet Account

```bash
# Generate a new keypair
stellar keys generate admin --network testnet

# Fund it with testnet XLM
stellar keys fund admin --network testnet
```

### 5b. Deploy Each Contract

```bash
# Build all contracts
stellar contract build

# Deploy confidential-token
CONFIDENTIAL_TOKEN=$(stellar contract deploy \
  --wasm contracts/confidential-token/target/soroban/confidential_token.wasm \
  --source admin \
  --network testnet)
echo "Confidential Token: $CONFIDENTIAL_TOKEN"

# Deploy verifier
VERIFIER=$(stellar contract deploy \
  --wasm contracts/verifier/target/soroban/verifier.wasm \
  --source admin \
  --network testnet)
echo "Verifier: $VERIFIER"

# Deploy auditor
AUDITOR=$(stellar contract deploy \
  --wasm contracts/auditor/target/soroban/auditor.wasm \
  --source admin \
  --network testnet)
echo "Auditor: $AUDITOR"

# Deploy auraflow-payroll (needs confidential token address)
PAYROLL=$(stellar contract deploy \
  --wasm contracts/auraflow-payroll/target/soroban/auraflow_payroll.wasm \
  --source admin \
  --network testnet)
echo "AuraFlow Payroll: $PAYROLL"
```

### 5c. Initialize Contracts

```bash
# Initialize confidential token with USDC as underlying
stellar contract invoke \
  --id $CONFIDENTIAL_TOKEN \
  --source admin \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --underlying_token CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

# Initialize payroll
stellar contract invoke \
  --id $PAYROLL \
  --source admin \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --confidential_token $CONFIDENTIAL_TOKEN \
  --usdc_contract CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA
```

### 5d. Record Deployed Addresses

Create `deployments/testnet.json`:
```json
{
  "network": "testnet",
  "admin": "<ADMIN_G_ADDRESS>",
  "contracts": {
    "confidential_token": "<DEPLOYED_ADDRESS>",
    "auraflow_payroll": "<DEPLOYED_ADDRESS>",
    "verifier": "<DEPLOYED_ADDRESS>",
    "auditor": "<DEPLOYED_ADDRESS>"
  },
  "underlying_usdc": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
  "usdc_issuer": "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5"
}
```

---

## Step 6: Update Environment Variables

New `.env.example`:
```bash
# Stellar Network
STELLAR_NETWORK_PASSPHRASE=Test SDF Future Network ; October 2022
STELLAR_RPC_URL=https://soroban-testnet.stellar.org

# Deployed Contract Addresses (fill after deployment)
CONFIDENTIAL_TOKEN_CONTRACT=
AURAFLOW_PAYROLL_CONTRACT=
VERIFIER_CONTRACT=
AUDITOR_CONTRACT=

# Underlying USDC on Stellar Testnet
USDC_CONTRACT=CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA
USDC_ISSUER=GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5

# Admin keypair (for deployment only, do not commit)
ADMIN_SECRET_KEY=
```

---

## Step 7: Update package.json

```json
{
  "name": "@auraflow/contracts",
  "version": "2.0.0",
  "description": "AuraFlow - Confidential Payroll on Stellar",
  "private": true,
  "scripts": {
    "build": "stellar contract build",
    "test": "soroban contract test",
    "deploy:testnet": "bash scripts/deploy-testnet.sh",
    "invoke": "stellar contract invoke"
  },
  "devDependencies": {
    "stellar-cli": ">=25.2"
  }
}
```

---

## Step 8: Testing

```bash
# Run unit tests
soroban contract test

# Test contract interactions on testnet
stellar contract invoke --id $CONFIDENTIAL_TOKEN --source admin --network testnet -- admin
```

---

## Key Differences from Solidity

| Aspect | Solidity (Arbitrum) | Soroban (Stellar) |
|--------|---------------------|-------------------|
| Language | Solidity | Rust |
| Build tool | Foundry | `stellar contract build` |
| Deploy tool | `forge script` | `stellar contract deploy` |
| Address format | `0x...` (20 bytes) | `C...` (32 bytes contract) / `G...` (32 bytes account) |
| Token standard | ERC-20 | Soroban Token (SEP-21) |
| Privacy | FHE stubs (Fhenix) | Pedersen commitments + UltraHonk ZK proofs |
| Streaming | Sablier LockupLinear | Custom PayrollVault (batch transfers) |
| Test framework | forge-std | `soroban contract test` |

---

## Reference Repositories

| Resource | URL |
|----------|-----|
| Lucent (ConfidentialToken) | https://github.com/ToluLabs/Lucent |
| OpenZeppelin Stellar Contracts | https://github.com/OpenZeppelin/stellar-contracts/tree/feat/confidential-verifier-ultrahonk |
| Nethermind UltraHonk Verifier | https://github.com/NethermindEth/rs-soroban-ultrahonk |
| Stellar Developer Preview | https://stellar.org/blog/developers/developer-preview-stellar-private-payments |
| Stellar Soroban Docs | https://soroban.stellar.org/docs |
