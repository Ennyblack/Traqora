# Wallet and payment timeouts (#v1)

When submitting blockchain transactions, network timeouts may leave the final outcome ambiguous.

## Rules
- Unknown outcomes are treated as `pending` or `timeout` state in the booking lifecycle.
- Bookings must undergo explicit reconciliation (`reconcile_timeout`) before subsequent payment attempts are permitted.
- Automated tests verify the normal path and timeout reconciliation flow.
