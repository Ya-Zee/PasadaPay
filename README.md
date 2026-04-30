# PasadaPay

> Automated boundary splitting for tricycle drivers and operators — powered by Stellar and Soroban.

---

## 📌 Overview

PasadaPay is a Soroban smart contract that automates the daily **boundary fee settlement** between tricycle drivers and operators in the Philippines.

Instead of manually handing over cash at the end of every shift — with no receipts and frequent disputes — the driver submits their gross earnings once through a mobile web app. The Soroban contract instantly calculates the pre-agreed split (e.g., 65% driver / 35% operator) and transfers XLM to both wallets in under 5 seconds for less than ₱0.01 in fees. Every payout is recorded on-chain as an immutable receipt.

---

## 🔴 Problem

A tricycle driver in Pasig City earns PHP 800–1,200/day but settles his daily boundary fee (PHP 300–400) in cash to the operator at the end of every shift. This process has no receipts, frequent disputes over amounts, and operators who pressure drivers to pay early before the shift ends. Drivers regularly lose PHP 50–150/day to undercounting or pressure — totalling PHP 1,500–4,500/month in lost income with zero recourse.

---

## ✅ Solution

PasadaPay lets the driver log his end-of-day gross earnings into a mobile web app. A Soroban smart contract:

1. Reads the pre-agreed boundary split ratio stored on-chain
2. Calculates each party's share to the stroop (1 XLM = 10,000,000 stroops)
3. Instantly transfers XLM to both the driver and operator wallets
4. Appends an immutable PayoutRecord to the on-chain history log

Settlement is sub-second. Fees are under ₱0.01. No bank, no GCash middleman, no disputes.

---

## ⚙️ Stellar Features Used

| Feature | Usage |
|---|---|
| XLM transfers | Earnings distributed to driver and operator wallets |
| Soroban smart contracts | Split ratio logic, auth enforcement, on-chain history |
| Trustlines | Optional: operator-issued BOUNDARY token for extended audit trail |

---

## 🎯 Target Users

| Role | Description |
|---|---|
| Drivers | Tricycle drivers in Metro Manila & Rizal province — daily income PHP 600–1,200 |
| Operators | Fleet owners with 2–20 units, collecting boundary in cash today |
| Location | Pasig, Marikina, Cainta, Antipolo — high-density tricycle corridors |

---

## 🚀 Core MVP Flow

```
Driver opens app
  → enters gross earnings (e.g., PHP 900)
  → app converts to XLM stroops at spot rate
  → calls submit_earnings(caller, gross_stroops)
  → contract verifies caller == registered driver
  → contract computes driver_amount = gross × 65 / 100
  → contract computes operator_amount = gross − driver_amount
  → XLM transferred to driver wallet (65%)
  → XLM transferred to operator wallet (35%)
  → PayoutRecord written to on-chain history
  → both wallets updated within 3–5 seconds
```

Demo-able end-to-end in under 90 seconds.

---

## 🧮 Split Math Reference

| Input | Value |
|---|---|
| Gross Earnings | PHP 900 → 9,000,000 stroops |
| Driver Share (65%) | 5,850,000 stroops ≈ PHP 585 |
| Operator / Boundary (35%) | 3,150,000 stroops ≈ PHP 315 |
| Transaction Fee | < 0.00001 XLM ≈ ₱0.001 |
| Settlement Time | 3–5 seconds (Stellar finality) |
| Daily Savings vs Cash | ₱50–150 saved per driver per day |

---

## 📁 Project Structure

```
PasadaPay/
├── Cargo.toml
└── src/
    ├── lib.rs       # Soroban smart contract
    └── test.rs      # 5 unit tests
```

---

## 🛠️ Prerequisites

- **Rust** (stable, >= 1.74)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-unknown-unknown
  ```

- **Soroban CLI** (>= 20.x)
  ```bash
  cargo install --locked soroban-cli
  ```

- **Stellar testnet account** funded via Friendbot
  ```bash
  curl "https://friendbot.stellar.org?addr=YOUR_PUBLIC_KEY"
  ```

---

## 🔨 Build

```bash
soroban contract build
```

Output: `target/wasm32-unknown-unknown/release/pasada_pay.wasm`

---

## ✅ Test

```bash
cargo test
```

Expected output:
```
test tests::test_happy_path_split_succeeds         ... ok
test tests::test_unauthorized_caller_rejected      ... ok
test tests::test_state_accumulates_across_multiple_submissions ... ok
test tests::test_zero_earnings_rejected            ... ok
test tests::test_update_split_changes_ratio        ... ok

test result: ok. 5 passed; 0 failed
```

---

## 🚢 Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pasada_pay.wasm \
  --source YOUR_SECRET_KEY \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
```

Save the returned `<CONTRACT_ID>` — you will need it for all subsequent invocations.

---

## ⚡ CLI Invocations

### 1. Initialize the Contract (run once)

Registers the driver-operator relationship and the agreed split ratio.

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source DRIVER_SECRET_KEY \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- init \
  --driver GDRIVER... \
  --operator GOPERATOR... \
  --xlm_token CNATIVE... \
  --driver_share_pct 65
```

---

### 2. Submit Daily Earnings (core MVP — run every shift)

Driver submits gross earnings in stroops. Contract splits and pays both wallets instantly.

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source DRIVER_SECRET_KEY \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- submit_earnings \
  --caller GDRIVER... \
  --gross_stroops 9000000
```

> 9,000,000 stroops = 0.9 XLM ≈ PHP 900 at current rates

---

### 3. View Payout History

Returns the full on-chain log of all past payouts.

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- get_history
```

---

### 4. View Current Split Config

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  -- get_config
```

---

### 5. View Lifetime Total Earnings

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  -- get_total_earnings
```

---

### 6. Update Split Ratio (requires both signatures)

Both driver and operator must sign. Prevents either party from unilaterally changing the terms.

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source DRIVER_SECRET_KEY \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- update_split \
  --driver GDRIVER... \
  --operator GOPERATOR... \
  --new_driver_pct 70
```

---

## 🔐 Contract Security Notes

- `submit_earnings` — only callable by the wallet registered as `driver` during `init`
- `update_split` — requires `require_auth()` from **both** driver and operator; neither party can change the ratio unilaterally
- `init` — can only be called once; re-initialization is blocked by an on-chain flag check
- All arithmetic uses integer stroop values — no floating point, no rounding errors

---

## 🌟 Optional Enhancements

| Enhancement | Description |
|---|---|
| SMS alerts | After each payout, emit a Soroban event → trigger SMS to operator's basic phone via Twilio or Semaphore PH |
| AI earnings estimator | Driver speaks gross earnings in Tagalog → AI parses and fills the form |
| Weekly summary report | Aggregate history log into a weekly PDF receipt for LTFRB compliance |
| Multi-driver support | Extend contract to manage a fleet: operator registers N drivers, each with individual split ratios |

---
## Deployed contract link
[1] https://stellar.expert/explorer/testnet/tx/cf643ec5499b0bc883491647172de46883f4823ff998cc4c24818b7c17f21d95
[2] https://lab.stellar.org/r/testnet/contract/CD7ZGLRMX7U3WN2SRLPPV4ERES5SSYQBTVUGZK5IGFCLE5POCC3TSO6L

## 📜 License
