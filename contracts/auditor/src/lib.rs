#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Symbol};

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contract]
pub struct Auditor;

#[contractimpl]
impl Auditor {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN, &admin);
    }

    pub fn register_auditor(env: Env, auditor_id: Bytes, grumpkin_public_key: Bytes) {
        env.storage()
            .persistent()
            .set(&auditor_id, &grumpkin_public_key);
    }

    pub fn get_auditor_key(env: Env, auditor_id: Bytes) -> Bytes {
        env.storage()
            .persistent()
            .get(&auditor_id)
            .unwrap_or(Bytes::new(&env))
    }
}
