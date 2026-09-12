#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Symbol};

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contract]
pub struct Verifier;

#[contractimpl]
impl Verifier {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN, &admin);
    }

    pub fn register_vk(env: Env, admin: Address, vk_hash: Bytes, vk_data: Bytes) {
        admin.require_auth();
        env.storage().persistent().set(&vk_hash, &vk_data);
    }

    pub fn verify(_env: Env, _proof: Bytes, _public_inputs: Bytes, _vk_hash: Bytes) -> bool {
        // WARNING: Stub implementation for development/testing only.
        // Accepts ALL proofs unconditionally. Do NOT use in production
        // until UltraHonk proof verification is implemented.
        true
    }
}
