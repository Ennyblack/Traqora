# Booking API v1 - Authorized Retrieval

Traqora v1 supports authorized booking detail retrieval via `get_booking(env, caller, booking_id)`.

## Authorization Rules
- The `caller` must provide valid Stellar contract authorization (`caller.require_auth()`).
- Access is strictly limited to either the booking's `passenger` or `airline`.
- Any unauthorized caller will cause the transaction to panic with `"Unauthorized booking access"`.
