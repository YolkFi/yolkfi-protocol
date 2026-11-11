use super::*;
use soroban_sdk::{testutils::Address as _, symbol_short, Address, Env};

#[test]
fn test_pull_accrual_mid_period_transfer_and_loss_absorption() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RevenueDistributor, ());
    let client = RevenueDistributorClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let lot_id = symbol_short!("LOTS1A");

    client.initialize(&admin);
    client.init_lot(&lot_id, &1_000); // 10% reserve

    // Mint 600 shares to Alice, 400 shares to Bob (total 1,000 shares)
    client.settle_balance_change(&lot_id, &None, &alice, &600);
    client.settle_balance_change(&lot_id, &None, &bob, &400);

    // Period 1: Deposit 10,000 EURC (100_000_000_000 stroops)
    // 10% reserve = 10_000_000_000 stroops, holder pool = 90_000_000_000 stroops
    client.deposit_revenue(&lot_id, &100_000_000_000);

    assert_eq!(client.accrued(&lot_id, &alice), 54_000_000_000);
    assert_eq!(client.accrued(&lot_id, &bob), 36_000_000_000);

    // Alice transfers 200 shares to Bob mid-cycle; Alice retains pre-transfer accrual
    client.settle_balance_change(&lot_id, &Some(alice.clone()), &bob, &200);
    assert_eq!(client.accrued(&lot_id, &alice), 54_000_000_000);
    assert_eq!(client.accrued(&lot_id, &bob), 36_000_000_000);

    // Period 2: Loss period (-5,000_000_000 stroops) absorbed by reserve; index never decreases
    let state_after_loss = client.deposit_revenue(&lot_id, &-5_000_000_000);
    assert_eq!(state_after_loss.reserve_balance, 5_000_000_000);
    assert_eq!(client.accrued(&lot_id, &alice), 54_000_000_000);

    // Period 3: Deposit another 100_000_000_000 stroops (now Alice=400, Bob=600)
    client.deposit_revenue(&lot_id, &100_000_000_000);
    // Alice: 54B + 36B = 90B; Bob: 36B + 54B = 90B
    assert_eq!(client.claim(&lot_id, &alice), 90_000_000_000);
    assert_eq!(client.claim(&lot_id, &bob), 90_000_000_000);

    // Invariant: total_claimed + reserve_balance + undistributed_dust == total_deposited - absorbed_loss
    let final_state = client.lot_state(&lot_id);
    assert!(final_state.total_claimed <= final_state.total_deposited);
}
