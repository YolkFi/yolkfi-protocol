use super::*;
use soroban_sdk::{testutils::Address as _, symbol_short, Address, BytesN, Env};

fn setup_registry(env: &Env) -> (FlockRegistryClient<'_>, Address, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(FlockRegistry, ());
    let client = FlockRegistryClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let operator = Address::generate(env);
    let protocol = Address::generate(env);
    let attestor = Address::generate(env);

    client.initialize(&admin, &operator, &protocol, &attestor);
    (client, admin, operator, protocol, attestor)
}

#[test]
fn test_register_three_lots_and_status_lifecycle() {
    let env = Env::default();
    let (client, _admin, _op, _proto, _attestor) = setup_registry(&env);

    let lot_1 = symbol_short!("LOTS1A");
    let lot_2 = symbol_short!("LOTS1B");
    let lot_3 = symbol_short!("LOTS1C");

    let doc_1 = BytesN::from_array(&env, &[1u8; 32]);
    let doc_2 = BytesN::from_array(&env, &[2u8; 32]);
    let doc_3 = BytesN::from_array(&env, &[3u8; 32]);

    client.register_lot(
        &lot_1,
        &symbol_short!("BARN01"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &30_000,
        &1_814_659_200,
        &30_000,
        &1_000,
        &doc_1,
    );
    client.register_lot(
        &lot_2,
        &symbol_short!("BARN02"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &25_000,
        &1_814_659_200,
        &25_000,
        &1_000,
        &doc_2,
    );
    client.register_lot(
        &lot_3,
        &symbol_short!("BARN03"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &20_000,
        &1_814_659_200,
        &20_000,
        &1_200,
        &doc_3,
    );

    let params = client.lot_params(&lot_1);
    assert_eq!(params.status, LotStatus::Placed);
    assert_eq!(params.share_supply, 30_000);
    assert_eq!(params.doc_root, doc_1);

    // Valid transitions: Placed -> Laying -> Molting -> Laying -> Depopulated
    assert_eq!(client.set_status(&lot_1, &LotStatus::Laying), LotStatus::Laying);
    assert_eq!(client.set_status(&lot_1, &LotStatus::Molting), LotStatus::Molting);
    assert_eq!(client.set_status(&lot_1, &LotStatus::Laying), LotStatus::Laying);
    assert_eq!(
        client.set_status(&lot_1, &LotStatus::Depopulated),
        LotStatus::Depopulated
    );
}

#[test]
fn test_reject_invalid_status_transitions() {
    let env = Env::default();
    let (client, _admin, _op, _proto, _attestor) = setup_registry(&env);

    let lot_id = symbol_short!("LOTS1A");
    let doc_root = BytesN::from_array(&env, &[9u8; 32]);
    client.register_lot(
        &lot_id,
        &symbol_short!("BARN01"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &30_000,
        &1_814_659_200,
        &30_000,
        &1_000,
        &doc_root,
    );

    // Placed -> Molting directly is invalid
    let res = client.try_set_status(&lot_id, &LotStatus::Molting);
    assert_eq!(res, Err(Ok(RegistryError::InvalidStatusTransition)));

    // Move to Laying -> Depopulated, then attempt to transition back to Laying
    client.set_status(&lot_id, &LotStatus::Laying);
    client.set_status(&lot_id, &LotStatus::Depopulated);
    let after_depop = client.try_set_status(&lot_id, &LotStatus::Laying);
    assert_eq!(after_depop, Err(Ok(RegistryError::InvalidStatusTransition)));
}

#[test]
fn test_allocation_gate_and_supply_cap() {
    let env = Env::default();
    let (client, _admin, _op, _proto, _attestor) = setup_registry(&env);

    let lot_id = symbol_short!("LOTS1A");
    let doc_root = BytesN::from_array(&env, &[7u8; 32]);
    client.register_lot(
        &lot_id,
        &symbol_short!("BARN01"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &1_000,
        &1_814_659_200,
        &1_000,
        &1_000,
        &doc_root,
    );

    let investor = Address::generate(&env);

    // Minting without prior allocation must be rejected
    let unallocated = client.try_mint_allocated(&lot_id, &investor, &100);
    assert_eq!(unallocated, Err(Ok(RegistryError::InsufficientAllocation)));

    // Allocation exceeding share_supply must be rejected
    let over_supply = client.try_authorise_allocation(&lot_id, &investor, &1_001);
    assert_eq!(over_supply, Err(Ok(RegistryError::SupplyExceeded)));

    // Valid allocation and mint
    assert_eq!(client.authorise_allocation(&lot_id, &investor, &600), 600);
    assert_eq!(client.mint_allocated(&lot_id, &investor, &400), 400);
    assert_eq!(client.allocation_of(&lot_id, &investor), 200);
    assert_eq!(client.lot_params(&lot_id).minted_supply, 400);
}

#[test]
fn test_early_termination_freezes_issuance() {
    let env = Env::default();
    let (client, _admin, _op, _proto, _attestor) = setup_registry(&env);

    let lot_id = symbol_short!("LOTS1A");
    let doc_root = BytesN::from_array(&env, &[5u8; 32]);
    client.register_lot(
        &lot_id,
        &symbol_short!("BARN01"),
        &symbol_short!("HNDRX"),
        &1_773_532_800,
        &30_000,
        &1_814_659_200,
        &30_000,
        &1_000,
        &doc_root,
    );

    let investor = Address::generate(&env);
    client.authorise_allocation(&lot_id, &investor, &500);

    let terminated = client.early_terminate(&lot_id);
    assert_eq!(terminated.status, LotStatus::Depopulated);
    assert!(terminated.terminated_early);

    // Minting after early termination must be rejected
    let res = client.try_mint_allocated(&lot_id, &investor, &500);
    assert_eq!(res, Err(Ok(RegistryError::LotTerminatedOrClosed)));
}
