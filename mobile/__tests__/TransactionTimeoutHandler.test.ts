import { TransactionTimeoutHandler } from '../booking/TransactionTimeoutHandler';

describe('TransactionTimeoutHandler', () => {
  beforeEach(() => {
    TransactionTimeoutHandler.setTimeoutThreshold(1000);
  });

  it('records submission and reports pending status', () => {
    const txId = 'tx-normal-1';
    const state = TransactionTimeoutHandler.recordSubmission(txId);
    expect(state.status).toBe('pending');
    expect(TransactionTimeoutHandler.checkStatus(txId)).toBe('pending');
  });

  it('allows initiating payment when no active pending transactions exist', () => {
    const canPay = TransactionTimeoutHandler.canInitiateNewPayment('G123', []);
    expect(canPay).toBe(true);
  });

  it('blocks new payment when a transaction is pending or timed out', () => {
    const txId = 'tx-blocked-1';
    TransactionTimeoutHandler.recordSubmission(txId);
    expect(TransactionTimeoutHandler.canInitiateNewPayment('G123', [txId])).toBe(false);
  });

  it('reconciles pending transaction to success and allows new payment', () => {
    const txId = 'tx-reconcile-1';
    TransactionTimeoutHandler.recordSubmission(txId);
    expect(TransactionTimeoutHandler.checkStatus(txId)).toBe('pending');

    TransactionTimeoutHandler.reconcile(txId, 'success');
    expect(TransactionTimeoutHandler.checkStatus(txId)).toBe('success');
    expect(TransactionTimeoutHandler.canInitiateNewPayment('G123', [txId])).toBe(true);
  });

  it('transitions to timeout status after threshold exceeded and prevents payment until reconciled', async () => {
    TransactionTimeoutHandler.setTimeoutThreshold(10);
    const txId = 'tx-timeout-1';
    TransactionTimeoutHandler.recordSubmission(txId);

    await new Promise((resolve) => setTimeout(resolve, 25));

    expect(TransactionTimeoutHandler.checkStatus(txId)).toBe('timeout');
    expect(TransactionTimeoutHandler.canInitiateNewPayment('G123', [txId])).toBe(false);

    TransactionTimeoutHandler.reconcile(txId, 'failed');
    expect(TransactionTimeoutHandler.canInitiateNewPayment('G123', [txId])).toBe(true);
  });
});
