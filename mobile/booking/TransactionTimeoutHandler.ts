export type TransactionOutcomeStatus = 'success' | 'failed' | 'pending' | 'timeout';

export interface TransactionSubmissionState {
  transactionId: string;
  status: TransactionOutcomeStatus;
  timestamp: number;
  retryCount: number;
}

export class TransactionTimeoutHandler {
  private static pendingTransactions: Map<string, TransactionSubmissionState> = new Map();
  private static timeoutThresholdMs: number = 45000;

  static setTimeoutThreshold(ms: number): void {
    TransactionTimeoutHandler.timeoutThresholdMs = ms;
  }

  static getTimeoutThreshold(): number {
    return TransactionTimeoutHandler.timeoutThresholdMs;
  }

  static recordSubmission(transactionId: string): TransactionSubmissionState {
    const state: TransactionSubmissionState = {
      transactionId,
      status: 'pending',
      timestamp: Date.now(),
      retryCount: 0,
    };
    TransactionTimeoutHandler.pendingTransactions.set(transactionId, state);
    return state;
  }

  static checkStatus(transactionId: string): TransactionOutcomeStatus {
    const state = TransactionTimeoutHandler.pendingTransactions.get(transactionId);
    if (!state) {
      return 'success';
    }
    if (state.status === 'pending') {
      const elapsed = Date.now() - state.timestamp;
      if (elapsed > TransactionTimeoutHandler.timeoutThresholdMs) {
        state.status = 'timeout';
      }
    }
    return state.status;
  }

  static reconcile(transactionId: string, resolvedStatus: 'success' | 'failed'): void {
    const state = TransactionTimeoutHandler.pendingTransactions.get(transactionId);
    if (state) {
      state.status = resolvedStatus;
      if (resolvedStatus === 'success' || resolvedStatus === 'failed') {
        TransactionTimeoutHandler.pendingTransactions.delete(transactionId);
      }
    }
  }

  static canInitiateNewPayment(walletAddress: string, activeTransactions: string[]): boolean {
    for (const txId of activeTransactions) {
      const status = TransactionTimeoutHandler.checkStatus(txId);
      if (status === 'pending' || status === 'timeout') {
        return false;
      }
    }
    return true;
  }
}
