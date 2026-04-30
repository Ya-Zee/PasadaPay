#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    /// Helper: deploy contract and initialize with a 65/35 split.
    fn setup() -> (Env, Address, Address, Address, PasadaPayClient<'static>) {
        let env = Env::default();
        env.mock_all_auths(); // auto-approve all auth checks in tests
        let driver = Address::generate(&env);
        let operator = Address::generate(&env);
        let xlm_token = Address::generate(&env); // mock XLM token address
        let contract_id = env.register_contract(None, PasadaPay);
        let client = PasadaPayClient::new(&env, &contract_id);
        // Initialize with 65% driver / 35% operator split
        client.init(&driver, &operator, &xlm_token, &65u32);
        (env, driver, operator, xlm_token, client)
    }

    // ── Test 1: Happy Path ────────────────────────────────────────────────────
    // Driver submits 1 XLM (10_000_000 stroops); contract splits and records payout.
    #[test]
    fn test_happy_path_split_succeeds() {
        let (env, driver, _, _, client) = setup();
        // 1 XLM = 10_000_000 stroops
        client.submit_earnings(&driver, &10_000_000i128);

        // Verify payout was recorded
        let history = client.get_history();
        assert_eq!(history.len(), 1);

        let record = history.get(0).unwrap();
        assert_eq!(record.gross, 10_000_000);
        // 65% of 10_000_000 = 6_500_000
        assert_eq!(record.driver_amount, 6_500_000);
        // 35% of 10_000_000 = 3_500_000
        assert_eq!(record.operator_amount, 3_500_000);
    }

    // ── Test 2: Edge Case — Unauthorized caller rejected ──────────────────────
    // An impostor (not the registered driver) tries to submit earnings.
    #[test]
    #[should_panic(expected = "only the registered driver can submit earnings")]
    fn test_unauthorized_caller_rejected() {
        let (env, _, _, _, client) = setup();
        let impostor = Address::generate(&env);
        // Impostor attempts to submit — should panic
        client.submit_earnings(&impostor, &10_000_000i128);
    }

    // ── Test 3: State Verification — total earnings and history accumulate ────
    // After two submissions, total_earnings and history length must both be correct.
    #[test]
    fn test_state_accumulates_across_multiple_submissions() {
        let (env, driver, _, _, client) = setup();
        client.submit_earnings(&driver, &9_000_000i128);  // Day 1: 0.9 XLM
        client.submit_earnings(&driver, &11_000_000i128); // Day 2: 1.1 XLM

        // Total should equal sum of both submissions
        assert_eq!(client.get_total_earnings(), 20_000_000i128);
        // History should have exactly 2 records
        assert_eq!(client.get_history().len(), 2);
    }

    // ── Test 4: Edge Case — zero earnings submission rejected ─────────────────
    #[test]
    #[should_panic(expected = "earnings must be greater than zero")]
    fn test_zero_earnings_rejected() {
        let (env, driver, _, _, client) = setup();
        client.submit_earnings(&driver, &0i128); // should panic
    }

    // ── Test 5: Update split requires both parties' authorization ─────────────
    // After update_split(75), driver's share in config must reflect 75%.
    #[test]
    fn test_update_split_changes_ratio() {
        let (env, driver, operator, _, client) = setup();
        client.update_split(&driver, &operator, &75u32);
        let config = client.get_config();
        assert_eq!(config.driver_share_pct, 75);

        // Verify next payout uses new ratio
        client.submit_earnings(&driver, &10_000_000i128);
        let record = client.get_history().get(0).unwrap();
        // 75% of 10_000_000 = 7_500_000
        assert_eq!(record.driver_amount, 7_500_000);
    }
}