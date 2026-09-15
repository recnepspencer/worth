import type { SignalValue } from "../model.js";
import type {
  ResourceLineDiagnosticsSummary,
  ResourceLineSummary,
} from "./resource_line_summary.js";
import type {
  ResourceMutationResponseConfirmationKind,
  ResourceMutationResponsePlan,
} from "./resource_mutation_response.js";
import type {
  ResourceLine,
  ResourceLineFreshness,
  ResourceLineFulfilledStatus,
  ResourceLineRejectedStatus,
  ResourceLineTimedOutStatus,
} from "./resource_lifecycle.js";

export interface ResourceLineAwaitSettlementOptions {
  /**
   * Deadline while waiting for this line's tip status to leave pending. When
   * it elapses first the promise resolves with `resultKind: "timedOut"` and a
   * `timedOut` status for the pending operation; it never rejects for time.
   * The line itself is unaffected: it stays pending and settles later, and a
   * new `awaitSettlement()` call observes that settlement. Not a paint
   * barrier — UI freshness follows tip notify.
   */
  readonly timeoutMs?: number;
  /**
   * When true, also drain root `settleAuthoredWork()` after tip status settles.
   * Opt-in tip-honest handoff; default false so awaitSettlement is not a global
   * mutation-queue wait used for UI freshness.
   */
  readonly drainAuthoredWork?: boolean;
}

export interface ResourceLineExecutionOptions {
  readonly freeOnSettle?: boolean;
}

export interface ResourceLineAwaitSettlementFulfilledResult<
  TParams = unknown,
  TValue = SignalValue,
> {
  readonly resultKind: "fulfilled" | "partial";
  readonly status: ResourceLineFulfilledStatus;
  readonly value: TValue;
  readonly summary: ResourceLineSummary<TParams>;
  readonly freshness: ResourceLineFreshness;
  readonly diagnosticsSummary: ResourceLineDiagnosticsSummary;
  readonly mutationResponse: ResourceMutationResponsePlan | null;
  readonly confirmationKind: ResourceMutationResponseConfirmationKind | null;
}

/**
 * `rejected` is the line's own settlement. `timedOut` is either the line's
 * policy timeout (the line status is `timedOut`) or the waiter's `timeoutMs`
 * deadline (the line status is still `pending`); `status.operation` names the
 * operation in both cases.
 */
export interface ResourceLineAwaitSettlementFailedResult<TParams = unknown> {
  readonly resultKind: "rejected" | "timedOut";
  readonly status: ResourceLineRejectedStatus | ResourceLineTimedOutStatus;
  readonly summary: ResourceLineSummary<TParams>;
  readonly freshness: ResourceLineFreshness;
  readonly diagnosticsSummary: ResourceLineDiagnosticsSummary;
  readonly mutationResponse: ResourceMutationResponsePlan | null;
  readonly confirmationKind: null;
}

export type ResourceLineAwaitSettlementResult<
  TParams = unknown,
  TValue = SignalValue,
> =
  | ResourceLineAwaitSettlementFulfilledResult<TParams, TValue>
  | ResourceLineAwaitSettlementFailedResult<TParams>;

export interface ResourceLineExecution<
  TParams = unknown,
  TValue = SignalValue,
> {
  readonly line: ResourceLine<TParams, TValue>;
  settled(
    options?: ResourceLineAwaitSettlementOptions,
  ): Promise<ResourceLineAwaitSettlementResult<TParams, TValue>>;
  free(): void;
  [Symbol.dispose](): void;
}
