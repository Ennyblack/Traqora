use integration_tests::{generate_actors, initialize_token, new_env, register_contracts};
use soroban_sdk::{testutils::Ledger, Symbol};


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
fn test_booking_list_pagination() {
    let env = new_env();
    let actors = generate_actors(&env);
    let contracts = register_contracts(&env);
    initialize_token(&env, &contracts.token, &actors.admin);

    let price = 100_0000000i128;

    // Create 3 bookings
    let id1 = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL101"),
        &Symbol::new(&env, "JFK"),
        &Symbol::new(&env, "LAX"),
        &1704067200,
        &price,
        &contracts.token.address,
    );
    let id2 = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL102"),
        &Symbol::new(&env, "LAX"),
        &Symbol::new(&env, "SFO"),
        &1704067300,
        &price,
        &contracts.token.address,
    );
    let id3 = contracts.booking.create_booking(
        &actors.passenger,
        &actors.airline,
        &Symbol::new(&env, "FL103"),
        &Symbol::new(&env, "SFO"),
        &Symbol::new(&env, "ORD"),
        &1704067400,
        &price,
        &contracts.token.address,
    );

    // Test limit 2, offset 0
    let page1 = contracts.booking.get_traveler_bookings(&actors.passenger, &2, &0);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap().booking_id, id1);
    assert_eq!(page1.get(1).unwrap().booking_id, id2);

    // Test limit 2, offset 2
    let page2 = contracts.booking.get_traveler_bookings(&actors.passenger, &2, &2);
    assert_eq!(page2.len(), 1);
    assert_eq!(page2.get(0).unwrap().booking_id, id3);

    // Test limit 5, offset 5 (out of range)
    let page3 = contracts.booking.get_traveler_bookings(&actors.passenger, &5, &5);
    assert_eq!(page3.len(), 0);

    // Test isolation between different passengers
    let other_passenger = soroban_sdk::Address::generate(&env);
    let other_bookings = contracts.booking.get_traveler_bookings(&other_passenger, &10, &0);
    assert_eq!(other_bookings.len(), 0);
}
