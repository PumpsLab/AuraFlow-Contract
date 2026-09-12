# AuraFlow Contracts

> Soroban smart contracts for AuraFlow — privacy-first payroll on Stellar.

![Soroban](https://img.shields.io/badge/Soroban-21-7B61FF) ![Rust](https://img.shields.io/badge/Rust-1.84+-CE422B) ![Stellar](https://img.shields.io/badge/Stellar--SDK-21-14B8E6)

## Overview

AuraFlow uses four Soroban smart contracts to enable confidential payroll on Stellar:

| Contract | Purpose |
|----------|---------|
| **ConfidentialToken** | Manages private payroll balances with deposit/withdraw to USDC |
| **AuraflowPayroll** | Treasury management and batch payroll processing |
| **Verifier** | ZK proof verification (stub for development) |
| **Auditor** | Auditor registry and compliance |

## Architecture

```
auraflow-contracts/
├── contracts/
│   ├── confidential-token/   # Private balance management
│   ├── auraflow-payroll/     # Treasury & payroll batch
│   ├── verifier/             # ZK proof verification
│   └── auditor/              # Auditor registry
├── deployments/              # Deployed contract addresses
│   └── testnet.json
├── Cargo.toml                # Workspace root
└── .env.example              # Environment template
```

## Prerequisites

- Rust >= 1.84
- `wasm32v1-none` target: `rustup target add wasm32v1-none`
- `stellar` CLI: https://developers.stellar.org/docs/tools/developer-tools
- Node.js >= 18 (for deployment scripts)

## Build

```bash
# Build all contracts
stellar contract build

# Build a specific contract
stellar contract build --package confidential-token
```

WASM artifacts are output to `target/wasm32v1-none/release/`.

## Test

```bash
# Run all contract tests
cargo test

# Test a specific contract
cargo test -p confidential-token
```

## Deploy to Testnet

```bash
# 1. Configure environment
cp .env.example .env
# Edit .env with your admin secret key

# 2. Deploy ConfidentialToken
stellar contract deploy \
  --wasm target/wasm32v1-none/release/confidential_token.wasm \
  --source-account admin \
  --network testnet

# 3. Deploy AuraflowPayroll
stellar contract deploy \
  --wasm target/wasm32v1-none/release/auraflow_payroll.wasm \
  --source-account admin \
  --network testnet

# 4. Initialize contracts
stellar contract invoke \
  --id <CONFIDENTIAL_TOKEN_ADDRESS> \
  --source-account admin \
  --network testnet \
  -- initialize \
  --admin <ADMIN_PUBLIC_KEY> \
  --underlying_token <USDC_CONTRACT_ADDRESS>

stellar contract invoke \
  --id <AURAFLOW_PAYROLL_ADDRESS> \
  --source-account admin \
  --network testnet \
  -- initialize \
  --admin <ADMIN_PUBLIC_KEY> \
  --confidential_token <CONFIDENTIAL_TOKEN_ADDRESS> \
  --usdc_contract <USDC_CONTRACT_ADDRESS>
```

## Contract Details

### ConfidentialToken

Manages private payroll balances. Employees can deposit USDC, make confidential transfers, and withdraw.

**Methods:**
- `initialize(admin, underlying_token)` — Set admin and USDC contract
- `register(env, address, amount)` — Register a new account
- `deposit(env, from, to, amount)` — Deposit USDC into private balance
- `confidential_transfer(env, from, to, amount)` — Transfer between private balances
- `withdraw(env, from, amount)` — Withdraw USDC to wallet
- `get_commitment(env, address)` — Get account balance commitment

### AuraflowPayroll

Treasury management for employers. Holds USDC and processes batch payroll payments.

**Methods:**
- `initialize(admin, confidential_token, usdc_contract)` — Set contracts
- `fund_treasury(env, employer)` — Record treasury funding
- `register_employee_vault(env, employer, employee)` — Register employee for payroll
- `process_payroll_batch(env, employer, recipients, amounts)` — Pay multiple employees
- `get_treasury_balance(env, employer)` — Check treasury balance
- `withdraw_treasury(env, employer, amount)` — Withdraw from treasury

### Verifier

ZK proof verification. **Currently a stub that accepts all proofs** — do not use in production.

### Auditor

Registry for auditor addresses and compliance checks.

## Deployed Contracts (Testnet)

See `deployments/testnet.json` for current addresses.

## License

MIT — see [LICENSE](LICENSE).
