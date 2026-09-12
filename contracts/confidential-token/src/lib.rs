#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol};
use soroban_sdk::token::TokenClient;

const ADMIN: Symbol = symbol_short!("ADMIN");
const UNDERLYING: Symbol = symbol_short!("UNDERLY");

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Registered(Address),
}

#[contract]
pub struct ConfidentialToken;

#[contractimpl]
impl ConfidentialToken {
    pub fn initialize(env: Env, admin: Address, underlying_token: Address) {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&UNDERLYING, &underlying_token);
    }

    pub fn register(env: Env, user: Address) {
        user.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Registered(user.clone()), &true);
    }

    /// Deposit: user sends USDC to this contract, internal balance credited.
    /// The user must have already sent USDC to this contract address via a
    /// standard Stellar payment before calling deposit.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        assert!(amount > 0, "Amount must be positive");

        let current = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(user.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user), &(current + amount));
    }

    /// Confidential transfer: moves real USDC from `from` to `to` via the
    /// underlying token contract, and updates internal balances.
    pub fn confidential_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        _proof: Bytes,
    ) {
        from.require_auth();
        assert!(amount > 0, "Amount must be positive");

        let from_balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_balance >= amount, "Insufficient private balance");

        // Transfer real USDC from `from` to `to` via the underlying token
        let underlying: Address = env.storage().instance().get(&UNDERLYING).unwrap();
        let token_client = TokenClient::new(&env, &underlying);
        token_client.transfer(&from, &to, &amount);

        // Update internal balances
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));

        let to_balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

    /// Withdraw: sends real USDC from this contract to the user's wallet.
    /// The user must have sufficient internal balance.
    pub fn withdraw(env: Env, user: Address, amount: i128, _proof: Bytes) -> bool {
        user.require_auth();
        assert!(amount > 0, "Amount must be positive");

        let current = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(user.clone()))
            .unwrap_or(0);
        assert!(current >= amount, "Insufficient private balance");

        // Transfer real USDC from this contract to the user
        let underlying: Address = env.storage().instance().get(&UNDERLYING).unwrap();
        let token_client = TokenClient::new(&env, &underlying);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, &user, &amount);

        // Update internal balance
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user), &(current - amount));
        true
    }

    pub fn get_commitment(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(user))
            .unwrap_or(0)
    }
}
