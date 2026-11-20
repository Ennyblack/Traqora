use integration_tests::{generate_actors, initialize_token, new_env, register_contracts};
use soroban_sdk::{testutils::Ledger, Symbol};

#[test]
fn test_transaction_submission_timeout_and_reconciliation() {
    let env = new_env();
    env.mock_all_auths();
    let (_admin, airline, user, token, booking_contract) = register_contracts(&env);
    initialize_token(&env, &token, &user);

    booking_contract.create_booking(
        &user,
        &airline,
        &Symbol::new(&env, "FL123"),
        &Symbol::new(&env, "JFK"),
        &Symbol::new(&env, "LAX"),
        &1704067200,
        &1000000000,
        &token,
    );

    booking_contract.pay_for_booking(&1);
    let b = booking_contract.get_booking(&1);
    assert_eq!(b.status, BookingStatus::PendingPayment);

    // Trying to pay again while pending should fail or be blocked by pending reconciliation status rule
    let res = booking_contract.try_pay_for_booking(&1);
    assert!(res.is_err());

    // Reconcile as failed/timed out back to Created state
    booking_contract.reconcile_booking(&1, &false);
    let b_retried = booking_contract.get_booking(&1);
    assert_eq!(b_retried.status, BookingStatus::Created);

    // Now pay successfully and reconcile as confirmed
    booking_contract.pay_for_booking(&1);
    booking_contract.reconcile_booking(&1, &true);
    let b_paid = booking_contract.get_booking(&1);
    assert_eq!(b_paid.status, BookingStatus::Paid);
}

#[test]
fn test_payment_escrow_flow() {
    let env = new_env();
    let actors = generate_actors(&env);
    let contracts = register_contracts(&env);
    initialize_token(&env, &contracts.token, &actors.admin);

    let price = 100_0000000i128; // 100 USDC

    // 1. Create booking
    let booking_id = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL123"),
        &Symbol::new(&env, "JFK"),
        &Symbol::new(&env, "LAX"),
        &1704067200,
        &price,
        &contracts.token.address,
    );

    let booking = contracts.booking.get_booking(&booking_id).unwrap();
    assert_eq!(booking.status, Symbol::new(&env, "pending"));
    assert_eq!(booking.amount_escrowed, 0);

    // 2. Mint tokens to passenger and Pay
    contracts
        .token
        .mint(&actors.admin, &actors.passenger, &price);
    contracts.booking.pay_for_booking(&booking_id);

    let booking = contracts.booking.get_booking(&booking_id).unwrap();
    assert_eq!(booking.status, Symbol::new(&env, "confirmed"));
    assert_eq!(booking.amount_escrowed, price);
    assert_eq!(contracts.token.balance_of(&actors.passenger), 0);
    assert_eq!(
        contracts.token.balance_of(&contracts.booking.address),
        price
    );
    assert_eq!(contracts.token.balance_of(&actors.airline), 0);

    // 3. Release escrow to the airline after fulfillment.
    contracts.booking.release_payment_to_airline(&booking_id);

    let booking = contracts.booking.get_booking(&booking_id).unwrap();
    assert_eq!(booking.status, Symbol::new(&env, "completed"));
    assert_eq!(booking.amount_escrowed, 0);
    assert_eq!(contracts.token.balance_of(&contracts.booking.address), 0);
    assert_eq!(contracts.token.balance_of(&actors.airline), price);
}

#[test]
fn test_refund_flow() {
    let env = new_env();
    env.ledger().set_timestamp(1700000000);

    let actors = generate_actors(&env);
    let contracts = register_contracts(&env);
    initialize_token(&env, &contracts.token, &actors.admin);

    let price = 100_0000000i128;
    let departure_time = 1705000000; // Far in the future

    let booking_id = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL123"),
        &Symbol::new(&env, "JFK"),
        &Symbol::new(&env, "LAX"),
        &departure_time,
        &price,
        &contracts.token.address,
    );

    contracts
        .token
        .mint(&actors.admin, &actors.passenger, &price);
    contracts.booking.pay_for_booking(&booking_id);

    // Refund passenger
    contracts.booking.refund_passenger(&booking_id);

    let booking = contracts.booking.get_booking(&booking_id).unwrap();
    assert_eq!(booking.status, Symbol::new(&env, "refunded"));
    assert_eq!(booking.amount_escrowed, 0);
}

#[test]
fn test_list_bookings_pagination() {
    let env = new_env();
    let actors = generate_actors(&env);
    let contracts = register_contracts(&env);
    initialize_token(&env, &contracts.token, &actors.admin);

    let price = 100_0000000i128;
    let id1 = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL1"),
        &Symbol::new(&env, "JFK"),
        &Symbol::new(&env, "LAX"),
        &1704067200,
        &price,
        &contracts.token.address,
    );
    let id2 = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL2"),
        &Symbol::new(&env, "LAX"),
        &Symbol::new(&env, "SFO"),
        &1704067200,
        &price,
        &contracts.token.address,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    let page = contracts.booking.list_bookings(&1, &10);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().booking_id, 1);
    assert_eq!(page.get(1).unwrap().booking_id, 2);

    let page_limited = contracts.booking.list_bookings(&1, &1);
    assert_eq!(page_limited.len(), 1);
    assert_eq!(page_limited.get(0).unwrap().booking_id, 1);
}
