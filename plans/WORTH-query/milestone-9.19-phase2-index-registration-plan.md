# Phase 2: indexed managed registration ownership

## Boundary review

Native now owns public and maintained root publication. Query's ordinary public
lookup still scans all retained entries in RegistryState.owned, downcasts every
index and charges the original request for that population. Repeated open uses
the same scan; unregister searches the Vec by issued id. The shared payer Vec
copies and retains historical grants until every other view also closes.

The actual reuse gate is ManagedIndexState::same_association plus its S/I type:
runtime authority, principal, full-generation scope, actual World incarnation,
installed compiled Index/Parent Arc identity and configured adapter Arc identity.
A routing key supplies no substitute for that gate. The immutable runtime catalog
admits only one actual Index/Parent per compiled managed-index purpose.

Use the already-compiled managed-index purpose identity from
access_contract().graph_obligations().identity(), which includes the Index name,
nominal Parent binding and complete read meaning. The ordinary parent_obligation()
identity describes its Query and omits the Parent binding; it is insufficient.
Combine purpose bytes with owner-issued runtime/incarnation and full EntityIds.
This fixed-width ordered key copies no names and never orders Rust TypeIds.
Hostile same-route candidates must still fail typed/Arc checks without replacing
the genuine receiver. A stale unregister id cannot remove a reopened route.

## Implementation plan

1. Define the registration route beside ManagedIndexState using the fields above.
   Keep same_association as the actual hit gate. No consumer API changes.
2. Replace ManagedIndex Vec ownership with one directory: ordered route->issued id
   and issued id->owned state/cleanup/key. Both mappings mutate atomically under
   the existing registry mutex. Ordinary/repeated lookup pays original-request
   indexed routing work, without visiting unrelated state or NativeCount entries.
   Bound node comparisons explicitly by the ordered node's eleven slots for each
   fixed-width route. Admit insert and replacement removal work before mutation
   as well; a repeated open pays only the lookup paths.
3. Pre-admit original entry backing for two full ordered nodes in each map before
   either insert. Grants belong to the actual indexed registration, not an
   accumulating payer list. Pre-admit work, identity, combined capacity and memory
   before replacing any old entry; failed admission preserves the existing owner.
4. Remove by the exact issued id. Carry the removed state/grant and any final empty
   map backing beyond the mutex, destroying backing before its last grant. Runtime
   shutdown drains into inline scratch bounded by the existing shared 64 capacity,
   clears both actual maps while all grants live, then disposes in issued-id order.
   Teardown allocates no scratch heap and creates no new capacity policy.
5. Keep NativeCount's current Vec as its own owner; remove the obsolete mixed enum,
   shared managed-index Vec/payer storage and its material admission helper.
   Combined registration capacity becomes views.len + indexes.len, without a scan.
   Unrelated derived views keep their current publication and admission semantics.
6. Qualify the public ordinary path against actual SDK invoices at increasing
   retained registrations while World branches and Native scope data stay fixed.
   Require logarithmic increments rather than the previous doubling scan slope;
   do not fit exact counts or add a second work ledger. Reuse public close/reopen,
   generation, foreign-runtime/branch and provider-version twins. Add finite owner
   controls for exact same-route hostile candidates, stale removal ids, failed
   admission and final backing release where existing courts cannot convict them.
7. Run focused locality/owner tests then the existing public managed group and
   mandatory boundary/context/format/cap/function checks. Independent qa-loop,
   qa-tests and code-quality-qa gate this slice before commit.

The 1,000/2,000 geometry chain, 50,000 unrelated geometry and retained-query
whole-path growth evidence remains required after this registration prerequisite.
This slice does not rewrite provider-catalog routing or unrelated derived views,
and it does not close Phase 2.

## Public evidence

The genuine guarded public prefix program passes lookup and repeated-open growth
beside 8, 16 and 32 retained registrations. All seven World branches and six Native
scope records exist before measurement; no source data changes between samples.
Original SDK query invoices are 59,350 / 60,439 / 61,528 and repeated-open invoices
8,418 / 9,507 / 10,596: both doubling increments remain 1,089 under a predeclared
50% slope headroom. These numbers are observations, not fitted test oracles.

A genuine second nominal binding to the same Parent Query/Index starts cold,
builds independently and survives closing the first binding. The actual shared 64
capacity refuses a 65th registration while preserving an existing warm target and
allowing exact reuse; releasing one slot admits retry. Closing a stale issued id
cannot remove a reopened successor. The full public managed group passes 35 tests.
The private metadata-owner control passes: a real zero-budget SDK reservation
refuses before mutation, the existing route/id/payload and its original charge
remain unchanged, funded retry inserts independently, and removal/disposal release
both actual backings and all original grants. It uses genuine runtime/incarnation
and resolved-scope data but opaque scalar storage, making no typed Source claim.
All 39 existing derived-view owner controls pass after the NativeCount ownership
separation. Formatting, caps, boundary/context and function scrutiny pass; the
independent reviewer gates the final commit.
