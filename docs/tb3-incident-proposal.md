# Terminal-Bench 3 Incident Proposal: Relational-World Publication Boundary

## Executive Summary

This proposal defines a single-defect benchmark incident targeting the
semantic boundary where Relational mutation settlement completes but World
product publication remains incomplete. The incident tests whether recovery
correctly adopts settled Relational evidence without re-executing the mutation,
advances the product head exactly once, and retains recovery authority until
publication actually succeeds.

**Recommendation**: This single incident provides substantial TB3 difficulty.
The boundary between lower-owner settlement and higher-owner publication
creates multiple attractive failure modes, and the correct repair requires
understanding cross-owner authority, evidence adoption, and idempotent
publication semantics. Access to upstream WORTH documentation helps with
vocabulary but does not reveal the exact recovery protocol.

---

## A. Banking Journey

### Scenario: Estate Disbursement with Publication Interruption

A bank specialist processes an estate disbursement (transferring funds from a
deceased customer's account to beneficiaries). The operation involves:

1. Specialist authenticates and obtains authorization grant
2. Disbursement mutation executes through Query
3. Relational commits the graph changes (journal entries, balance updates)
4. Before World product publication completes, an interruption occurs
5. Recovery must complete the publication without re-executing Relational work

### Exact API Call Sequence

```text
Phase 1: Setup and Authentication
─────────────────────────────────
1. BankIdentityRuntime::authenticate(specialist_credential)
   → BankAuthenticatedPrincipal

2. BankIdentityRuntime::query(queries::estate_case(estate_id))
   .as_principal(&specialist)
   .controls(BankReadControls::current(...))
   .execute()
   → Estate case with death notice status

Phase 2: Disbursement Execution (Interrupted)
─────────────────────────────────────────────
3. BankIdentityRuntime::disburse_estate(
       &specialist,
       EstateAction::DisburseEstate { estate, beneficiary, amount },
       idempotency_key,
       &request_scope
   )
   → Relational commits, but World publication fails
   → BankMutationCommitOutcome with recovery handle

Phase 3: Recovery Inspection
────────────────────────────
4. BankIdentityRuntime::open_commit_recovery(&receipt)
   → BankCommitRecoveryHandle

5. BankIdentityRuntime::inspect_commit_recovery(
       &handle,
       &specialist,
       action,
       &request_scope
   )
   → BankRecoveryInspection showing:
      - cause: SettlementPending or ProductPublicationLost
      - next_actions: includes StartFreshCompositePublication
      - relational_posture: Settled

Phase 4: Recovery Completion
────────────────────────────
6. BankIdentityRuntime::resolve_commit_recovery(
       handle,
       &specialist,
       action,
       &request_scope
   )
   → If external effect: safe_retry or reconcile first
   → World adopts settled evidence, advances product head once

Phase 5: Verification
─────────────────────
7. BankIdentityRuntime::query(queries::account_summary(estate_account))
   → Balance reflects the disbursement exactly once

8. BankIdentityRuntime::account_activity(estate_account)
   → Journal entry appears exactly once

9. Retry of original disbursement with same idempotency key
   → AlreadyCommitted (not a new mutation)
```

### Key Accounts and Identities

| Entity | Role | Purpose |
|--------|------|---------|
| `estate_account` | Source | Deceased customer's account |
| `beneficiary_account` | Destination | Receives disbursement |
| `specialist` | Actor | Authorized estate processor |
| `deceased` | Subject | Original account owner |

---

## B. Incident Descriptions (Solver-Facing Draft)

### Incident TB3-RW-001: Incomplete Publication Recovery

**Setup**: A Bank World instance with:
- An estate case ready for disbursement
- A specialist with disbursement authority
- External effect transport installed (for estate notification dispatch)

**Operation Sequence**:
1. Authenticate specialist
2. Execute estate disbursement mutation
3. Observe that Relational graph changes committed successfully
4. Observe that World product publication did not complete (simulated
   interruption or publication race loss)
5. Open recovery handle from the commit receipt
6. Inspect recovery to observe available next actions
7. Attempt to resolve or complete recovery
8. Verify account balances and activity history
9. Retry the original operation with the same idempotency key

**Observed Incorrect Behavior** (in poisoned starter):
- Recovery cleanup removes the record before publication completes
- OR: Resolution re-executes the Relational mutation, creating duplicate
  journal entries
- OR: Product head advances but recovery record is released prematurely,
  leaving inconsistent state on crash
- OR: Retry after apparent completion creates a second disbursement

**Required Outcome**:
1. The Relational mutation executes exactly once across all attempts
2. Recovery completion advances the product head exactly once
3. The recovery handle remains valid until World publication succeeds
4. Cleanup is denied while the publication handoff is incomplete
5. Historical balance and activity reads reflect exactly one disbursement
6. Equivalent idempotent retries return `AlreadyCommitted` with the original
   receipt
7. Invalid recovery attempts (wrong principal, expired, foreign handle) are
   denied before any state change

### Acceptance Criteria Combinations

A solver must handle all combinations correctly:

| Relational State | World State | Recovery Action | Expected Outcome |
|------------------|-------------|-----------------|------------------|
| Performed (unsettled) | Unpublished | Cleanup | DENIED: SettlementRequired |
| Settled | Unpublished | Cleanup | DENIED: Publication incomplete |
| Settled | Unpublished | StartFreshPublication | Product head advances once |
| Settled | Published | Cleanup | ALLOWED |
| Settled | Published | Retry operation | AlreadyCommitted |

---

## C. Source Mutations for Poisoned Starter

### Target Files and Defect Nature

The clean baseline at commit `43b7d0e58872eaa736628e77889baa5df8e77dc8`
contains the correct recovery design. The poisoned starter introduces a subtle
semantic defect in one of these locations:

#### Option 1: Premature Cleanup Eligibility (Recommended)

**File**: `crates/worth-runtime-world/src/recovery/cleanup.rs`

**Defect**: Modify `cleanup_record` to check only `settlement_required()` and
not verify that World product publication has completed. The existing tests for
settlement denial pass, but the new incident's publication-incomplete case
fails.

```rust
// POISONED: Missing check for publication completion
pub(crate) fn cleanup_record(
    &self,
    handle: &super::ProductUnpublishedRecoveryHandle,
    age: Option<(crate::lifecycle::RuntimeWorldInstant, u64)>,
) -> Result<RecoveryCleanupOutcome, RecoveryCleanupDenial> {
    // ...existing code...
    let removed = self.remove_record_if_exclusive(handle, |record| {
        if record.settlement_required() {
            return false;
        }
        // MISSING: Should also check that StartFreshCompositePublication
        // has been consumed or that product head actually advanced
        // ...
    });
}
```

**Why existing tests pass**: Current phase8 tests exercise settlement denial
and safe-retry, but don't specifically test cleanup denial when Relational is
settled but World publication is incomplete.

#### Option 2: Duplicate Relational Execution

**File**: `crates/worth-runtime-world/src/lifecycle/owner/recovery_service.rs`

**Defect**: In `continue_effects`, instead of adopting the settled evidence,
re-invoke Relational settlement unnecessarily, which could create duplicate
effects if the settlement is not idempotent at that layer.

```rust
// POISONED: Re-executes settlement instead of adopting
fn continue_effects(&self, effects: ProductUnpublishedOwnerEffects)
    -> Result<RecoveryContinuationContract, ...>
{
    // ...existing code...
    if settlement_required {
        // CORRECT: adopt existing settlement
        // POISONED: re-call settlement port, risking duplicate
    }
}
```

#### Option 3: Evidence Discard on Publication Reservation

**File**: `workspaces/worth-query/crates/worth-query-execution/src/basis/product_branch/creation_recovery.rs`

**Defect**: When `StartFreshCompositePublication` is invoked, discard the
Relational evidence instead of adopting it, requiring the caller to re-execute
the entire operation.

### Existing Test Preservation

The poisoned starter must keep these tests green:

- `phase8_cross_gate::lost_response_recovery_through_real_rail_and_aftermath`
- `phase8_safe_retry::*` - all safe-retry tests
- `phase8_recovery_counters::mint_and_repeat_inspection_leave_recovery_work_at_zero`
- `phase8_exact_handle_authority::*` - all handle authority tests
- `ordinary_mutations::public_consumer_executes_every_typed_mutation_family`
- `activity_history::*` - all historical read tests

The new incident regression tests fail because they specifically exercise the
Relational-settled-but-World-unpublished boundary.

---

## D. Verifier Sketch

### Test Structure

```rust
#[test]
fn relational_settled_world_unpublished_recovery_completes_publication_exactly_once() {
    // Setup: Estate disbursement world with fault injection
    let world = disbursement_world_with_publication_fault("tb3-boundary");
    
    // Phase 1: Execute mutation that settles Relational but fails World
    world.inject_publication_fault(PublicationFault::ProductHeadRaceLoss);
    let receipt = world.commit_disbursement(amount, idempotency_key);
    
    // Verify: Relational settled, World unpublished
    assert!(receipt.relational_settled());
    assert!(!receipt.product_published());
    
    // Phase 2: Recovery inspection
    let handle = world.open_recovery(&receipt);
    let inspection = world.inspect_recovery(&handle);
    assert!(inspection.next_actions().contains(
        &StartFreshCompositePublication
    ));
    
    // Phase 3: Recovery completion
    world.clear_publication_fault();
    let continuation = world.continue_recovery(&handle);
    assert!(continuation.actions().contains(&ReleaseObligations));
    
    // Phase 4: Verification - mutation occurred exactly once
    let balance = world.query_balance(estate_account);
    assert_eq!(balance, original_balance - amount);
    
    let activity = world.query_activity(estate_account);
    assert_eq!(activity.journal_count(), 1);
    
    // Phase 5: Idempotent retry
    let retry = world.retry_disbursement(amount, idempotency_key);
    assert!(matches!(retry.status(), BankMutationStatus::AlreadyCommitted(_)));
}

#[test]
fn cleanup_denied_while_publication_incomplete() {
    let world = disbursement_world_with_publication_fault("tb3-cleanup-denial");
    world.inject_publication_fault(PublicationFault::ProductHeadRaceLoss);
    let receipt = world.commit_disbursement(amount, key);
    
    // Relational settled but World unpublished
    let handle = world.open_recovery(&receipt);
    
    // Cleanup must be denied
    let cleanup_result = world.cleanup_recovery(&handle);
    assert!(matches!(
        cleanup_result,
        Err(RecoveryCleanupDenial::PublicationIncomplete)
    ));
    
    // Complete publication
    world.clear_publication_fault();
    world.continue_recovery(&handle);
    
    // Now cleanup succeeds
    let cleanup_result = world.cleanup_recovery(&handle);
    assert!(cleanup_result.is_ok());
}

#[test]
fn repeated_recovery_does_not_duplicate_mutation() {
    let world = disbursement_world_with_publication_fault("tb3-no-duplicate");
    world.inject_publication_fault(PublicationFault::ProductHeadRaceLoss);
    let receipt = world.commit_disbursement(amount, key);
    
    // Multiple recovery attempts
    for _ in 0..3 {
        let handle = world.open_recovery(&receipt);
        let _ = world.continue_recovery(&handle);
    }
    
    // Clear fault and complete
    world.clear_publication_fault();
    let handle = world.open_recovery(&receipt);
    world.continue_recovery(&handle);
    
    // Only one journal entry exists
    let activity = world.query_activity(estate_account);
    assert_eq!(activity.journal_count(), 1);
}
```

### Independent Oracles

1. **Journal Count Oracle**: Query `account_activity` and count distinct
   journal entries for the operation
2. **Balance Oracle**: Query `account_summary` and verify exact balance change
3. **Idempotency Oracle**: Retry with same key, expect `AlreadyCommitted`
4. **Recovery Handle Validity**: Inspect handle after various operations

### Denial Cases

- Foreign principal recovery inspection
- Expired authentication recovery attempt
- Foreign handle cleanup
- Cross-runtime handle use
- Cleanup before settlement
- Cleanup before publication

---

## E. Difficulty Calibration

### Why Capable Agents Fail

1. **Attractive MVCC-only explanations**: A solver may focus on Relational's
   MVCC semantics (which are well-documented) and miss that the boundary
   involves World's product publication as a separate authority. The
   Relational commit being successful does not mean the operation is complete.

2. **Stale-but-true basis confusion**: The Relational evidence IS true and
   settled, but using it requires a fresh World publication reservation. A
   solver might incorrectly conclude that the settled evidence is sufficient
   authority to declare completion.

3. **Second authority temptation**: The obvious "fix" is to add a completion
   registry that tracks which operations finished. This creates a second
   authority that can disagree with actual publication state.

4. **Coordinator compensation**: A solver might add cleanup logic that "fixes"
   the platform boundary by detecting and re-executing incomplete operations,
   which violates the exactly-once contract.

5. **Evidence adoption vs. re-execution**: The correct repair adopts the
   settled evidence without re-contacting Relational. A solver might
   incorrectly retry the Relational leg, potentially creating duplicates.

### Upstream WORTH Access Assessment

- **Helps**: Understanding the vocabulary (ProductUnpublished, settlement,
  publication, recovery handles, next actions)
- **Helps**: Understanding the phase progression model
- **Does not help**: The exact protocol for evidence adoption is internal to
  the recovery service implementation
- **Does not help**: The specific denial conditions for cleanup are not in
  public documentation

### Estimated Difficulty

**High**. The incident requires:
- Understanding that Relational and World are separate owners with separate
  authority
- Understanding that settlement != publication
- Understanding the recovery handle lifecycle
- Correctly implementing evidence adoption without re-execution
- Correctly gating cleanup on actual publication completion

---

## F. Recommendation

### Single Incident Sufficiency

**This single incident is sufficient for TB3 difficulty.** The boundary
between Relational settlement and World publication is a real semantic
boundary in WORTH that requires understanding multiple authority owners and
their coordination protocol. The repair is non-obvious and multiple attractive
incorrect repairs exist.

### Composition Considerations (If Needed)

If additional difficulty is required, consider composing with:

1. **Concurrent publication race**: Two attempts racing to publish the same
   settled evidence, where exactly one must succeed
2. **External effect reconciliation**: The estate disbursement includes an
   external notification that must also be reconciled
3. **Signal dependency**: Adding a Signal policy evaluation that must also be
   preserved across the boundary

However, the recommended approach is to validate this single incident first
before adding complexity.

---

## Author Notes (Not Solver-Facing)

### Intended Repair Algorithm

> World validates the live recovery record, reserves a fresh product
> publication, adopts the settled Relational evidence without contacting
> Relational again, and advances the product head once. Query retains both
> the recovery record and suspension completion until that World movement
> actually happens.

### Key Implementation Points

1. `ProductUnpublishedOwnerEffects::next_actions()` includes
   `StartFreshCompositePublication` only when Relational evidence is settled
2. The fresh publication adopts the settled evidence via
   `CompositeAttemptProgress` with `RelationalAttemptProgressPosture::Settled`
3. Cleanup checks both `settlement_required()` AND publication completion
4. The recovery handle remains valid across the entire lifecycle
5. Idempotency resolution happens at the Query layer, not Bank layer

### Provenance

- Clean baseline: commit `43b7d0e58872eaa736628e77889baa5df8e77dc8`
- Key files examined:
  - `crates/worth-runtime-world/src/recovery/cleanup.rs`
  - `crates/worth-runtime-world/src/recovery/product_unpublished/actions.rs`
  - `crates/worth-runtime-world/src/lifecycle/owner/recovery_service.rs`
  - `workspaces/worth-query-bank-world/crates/bank-server/tests/ordinary_mutations/estate_operations/phase8_*.rs`
- No existing fix commits found; the clean baseline already contains the
  correct design
