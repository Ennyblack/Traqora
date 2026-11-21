#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Symbol, symbol_short};

#[test]
fn test_timeout_reconciliation_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BookingContract, ());
    let client = BookingContractClient::new(&env, &contract_id);

    let passenger = Address::generate(&env);
    let airline = Address::generate(&env);
    let token = Address::generate(&env);

    let booking_id = client.create_booking(
        &passenger,
        &airline,
        &symbol_short!("FL123"),
        &symbol_short!("JFK"),
        &symbol_short!("LAX"),
        &1000000,
        &500,
        &token,
    );

    let booking = client.get_booking(&booking_id).unwrap();
    assert_eq!(booking.status, symbol_short!("pending"));

    // Simulate timeout status setting
    BookingContract::set_status(env.clone(), booking_id, symbol_short!("timeout"));
    let booking_timeout = client.get_booking(&booking_id).unwrap();
    assert_eq!(booking_timeout.status, symbol_short!("timeout"));

    // Reconcile as not confirmed (goes back to pending)
    client.reconcile_timeout(&booking_id, &false);
    let booking_pending = client.get_booking(&booking_id).unwrap();
    assert_eq!(booking_pending.status, symbol_short!("pending"));

    // Reconcile as confirmed
    BookingContract::set_status(env.clone(), booking_id, symbol_short!("timeout"));
    client.reconcile_timeout(&booking_id, &true);
    let booking_confirmed = client.get_booking(&booking_id).unwrap();
    assert_eq!(booking_confirmed.status, symbol_short!("confirmed"));
}
