#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

// ── Storage Keys ──────────────────────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Config,               // SplitConfig — driver, operator, ratio
    History,              // Vec<PayoutRecord> — audit log
    TotalEarnings,        // i128 — cumulative gross submitted
}

// ── Data Structures ───────────────────────────────────────────────────────────

/// Stores the permanent relationship between driver and operator, and their agreed split.
/// driver_share_pct: integer 1–99 representing the driver's percentage (e.g. 65 = 65%).
#[contracttype]
#[derive(Clone)]
pub struct SplitConfig {
    pub driver: Address,
    pub operator: Address,
    pub xlm_token: Address,      // native XLM token contract address on Stellar
    pub driver_share_pct: u32,   // e.g. 65 means driver gets 65%, operator gets 35%
}

/// Immutable record of a single payout event — stored in on-chain history.
#[contracttype]
#[derive(Clone)]
pub struct PayoutRecord {
    pub ledger: u32,             // ledger sequence at time of payout
    pub gross: i128,             // total XLM submitted (in stroops: 1 XLM = 10_000_000)
    pub driver_amount: i128,     // XLM paid to driver
    pub operator_amount: i128,   // XLM paid to operator
}

// ── Contract ──────────────────────────────────────────────────────────────────
#[contract]
pub struct PasadaPay;

#[contractimpl]
impl PasadaPay {
    /// Called once by the driver (or a setup admin) to register the boundary agreement.
    /// driver_share_pct must be between 1 and 99 inclusive.
    pub fn init(
        env: Env,
        driver: Address,
        operator: Address,
        xlm_token: Address,
        driver_share_pct: u32,
    ) {
        // Prevent re-initialization
        assert!(
            !env.storage().instance().has(&DataKey::Config),
            "contract already initialized"
        );
        assert!(
            driver_share_pct >= 1 && driver_share_pct <= 99,
            "driver_share_pct must be 1–99"
        );
        driver.require_auth(); // driver must authorize setup

        env.storage().instance().set(
            &DataKey::Config,
            &SplitConfig { driver, operator, xlm_token, driver_share_pct },
        );
        env.storage().instance().set(&DataKey::TotalEarnings, &0i128);
        env.storage().instance().set(&DataKey::History, &Vec::<PayoutRecord>::new(&env));
    }

    /// Driver submits gross daily earnings in stroops (1 XLM = 10_000_000 stroops).
    /// Contract calculates split and instantly transfers XLM to both wallets.
    /// Only the registered driver may call this function.
    pub fn submit_earnings(env: Env, caller: Address, gross_stroops: i128) {
        caller.require_auth();

        let config: SplitConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        assert!(caller == config.driver, "only the registered driver can submit earnings");
        assert!(gross_stroops > 0, "earnings must be greater than zero");

        // ── Calculate split ──────────────────────────────────────────────────
        // driver_amount = gross * driver_share_pct / 100
        // operator_amount = gross - driver_amount  (handles any rounding remainder)
        let driver_amount = gross_stroops * config.driver_share_pct as i128 / 100;
        let operator_amount = gross_stroops - driver_amount;

        // ── Transfer XLM from driver to contract, then split out ─────────────
        let token_client = token::Client::new(&env, &config.xlm_token);

        // Move gross from driver wallet into contract
        token_client.transfer(
            &config.driver,
            &env.current_contract_address(),
            &gross_stroops,
        );

        // Pay driver their share back immediately
        token_client.transfer(
            &env.current_contract_address(),
            &config.driver,
            &driver_amount,
        );

        // Pay operator their boundary share
        token_client.transfer(
            &env.current_contract_address(),
            &config.operator,
            &operator_amount,
        );

        // ── Update cumulative earnings ────────────────────────────────────────
        let mut total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalEarnings)
            .unwrap_or(0);
        total += gross_stroops;
        env.storage().instance().set(&DataKey::TotalEarnings, &total);

        // ── Append to history log ─────────────────────────────────────────────
        let mut history: Vec<PayoutRecord> =
            env.storage().instance().get(&DataKey::History).unwrap();
        history.push_back(PayoutRecord {
            ledger: env.ledger().sequence(),
            gross: gross_stroops,
            driver_amount,
            operator_amount,
        });
        env.storage().instance().set(&DataKey::History, &history);
    }

    /// Update the split ratio. Both driver and operator must authorize this call
    /// to prevent unilateral changes by either party.
    pub fn update_split(env: Env, driver: Address, operator: Address, new_driver_pct: u32) {
        driver.require_auth();
        operator.require_auth();
        assert!(
            new_driver_pct >= 1 && new_driver_pct <= 99,
            "driver_share_pct must be 1–99"
        );
        let mut config: SplitConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        assert!(driver == config.driver && operator == config.operator, "unauthorized");
        config.driver_share_pct = new_driver_pct;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    /// Returns the current split configuration — for UI display.
    pub fn get_config(env: Env) -> SplitConfig {
        env.storage().instance().get(&DataKey::Config).unwrap()
    }

    /// Returns lifetime total gross earnings submitted through this contract.
    pub fn get_total_earnings(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalEarnings).unwrap_or(0)
    }

    /// Returns the full payout history log.
    pub fn get_history(env: Env) -> Vec<PayoutRecord> {
        env.storage().instance().get(&DataKey::History).unwrap()
    }
}