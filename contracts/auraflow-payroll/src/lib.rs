#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};
use soroban_sdk::token::TokenClient;

const ADMIN: Symbol = symbol_short!("ADMIN");
const CONFIDENTIAL_TOKEN: Symbol = symbol_short!("CON_TOK");
const USDC: Symbol = symbol_short!("USDC");

#[contracttype]
pub enum DataKey {
    TreasuryBalance(Address, Address),
    EmployeeVault(Address),
    EmployeeEmployer(Address),
}

#[contract]
pub struct PayrollVault;

#[contractimpl]
impl PayrollVault {
    pub fn initialize(
        env: Env,
        admin: Address,
        confidential_token: Address,
        usdc_contract: Address,
    ) {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage()
            .instance()
            .set(&CONFIDENTIAL_TOKEN, &confidential_token);
        env.storage().instance().set(&USDC, &usdc_contract);
    }

    pub fn fund_treasury(env: Env, employer: Address, amount: i128) {
        employer.require_auth();
        assert!(amount > 0, "Amount must be positive");

        let usdc: Address = env.storage().instance().get(&USDC).unwrap();
        let current = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::TreasuryBalance(employer.clone(), usdc.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::TreasuryBalance(employer, usdc),
            &(current + amount),
        );
    }

    pub fn register_employee_vault(env: Env, employee: Address) {
        employee.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::EmployeeVault(employee.clone()), &true);
    }

    /// Process a batch of payroll payments.
    /// Deducts from employer's treasury balance and transfers real USDC
    /// from this contract to each recipient.
    pub fn process_payroll_batch(
        env: Env,
        employer: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) {
        employer.require_auth();
        assert!(recipients.len() == amounts.len(), "Mismatched arrays");

        let usdc: Address = env.storage().instance().get(&USDC).unwrap();
        let mut total_amount: i128 = 0;
        for i in 0..amounts.len() {
            total_amount += amounts.get(i).unwrap();
        }

        let balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::TreasuryBalance(employer.clone(), usdc.clone()))
            .unwrap_or(0);
        assert!(balance >= total_amount, "Insufficient treasury balance");

        // Deduct from employer's internal treasury balance
        env.storage().persistent().set(
            &DataKey::TreasuryBalance(employer, usdc.clone()),
            &(balance - total_amount),
        );

        // Transfer real USDC from this contract to each recipient
        let contract_address = env.current_contract_address();
        let token_client = TokenClient::new(&env, &usdc);

        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            token_client.transfer(&contract_address, &recipient, &amount);
        }
    }

    pub fn get_treasury_balance(env: Env, employer: Address) -> i128 {
        let usdc: Address = env.storage().instance().get(&USDC).unwrap();
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::TreasuryBalance(employer, usdc))
            .unwrap_or(0)
    }

    /// Withdraw USDC from the treasury back to the employer's wallet.
    pub fn withdraw_treasury(env: Env, employer: Address, amount: i128) {
        employer.require_auth();
        assert!(amount > 0, "Amount must be positive");

        let usdc: Address = env.storage().instance().get(&USDC).unwrap();
        let balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::TreasuryBalance(employer.clone(), usdc.clone()))
            .unwrap_or(0);
        assert!(balance >= amount, "Insufficient treasury balance");

        // Deduct from internal balance
        env.storage().persistent().set(
            &DataKey::TreasuryBalance(employer.clone(), usdc.clone()),
            &(balance - amount),
        );

        // Transfer real USDC from this contract to the employer
        let contract_address = env.current_contract_address();
        let token_client = TokenClient::new(&env, &usdc);
        token_client.transfer(&contract_address, &employer, &amount);
    }
}
