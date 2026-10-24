# Database Foundation Roadmap

Make Store the durable foundation for Relational and the production application
host. Preserve the [Store vision](../worth-store/worth_store_vision.md):
canonical commits, structural deltas, branch history, lineage, schema evolution
and retention remain meaningful across crashes and restarts.

## Milestone 1: Canonical durable commits

Store can durably publish and recover a complete semantic commit.

Requirements:

- Persist canonical commit envelopes and their structural changes together
  with the selected successor state. Physical layouts and indexes preserve
  canonical meaning and remain reconstructible within retained coverage.
- Support atomic batch publication through the existing Store media, WAL,
  integrity, checkpoint and recovery owners.
- Establish one durable commit decision. A crash cannot expose a mixture of
  the previous commit and its successor.
- Preserve exact commit and attempt identities so an ambiguous submission can
  be resolved without submitting the same effect again.
- Recover both before the first checkpoint and from a checkpoint plus its
  bounded journal tail.
- Distinguish absent data, unsupported formats, corruption, proven no effect
  and indeterminate effects. Existing damaged data must not be initialized as
  a new store.
- Support changed-path persistence and shared immutable data so later branches
  and cold reads can use this same storage path.

Complete when real filesystem crash tests recover the exact committed
artifacts and selected state, or the exact unresolved fate, at the publication
boundaries. Physical completion must not be mistaken for application success.

## Milestone 2: Relational cutover to Store

Relational uses Store for durable committed state while retaining ownership
of graph semantics, transaction validation, identity and MVCC.

Requirements:

- Connect ordinary prepared Relational transactions to Store's durable commit
  boundary.
- Persist and recover records, relations, aspects, schema, identity allocation,
  lineage and canonical commit history needed by the supported transactions.
- Reconstruct the original logical identities through their owning runtime;
  issue fresh live capabilities after restart.
- Preserve validation, immutable observations, stale-candidate refusals and
  typed publication outcomes.
- Introduce storage access that supports cold data and guarded native working
  state. The durable database must not depend on a second authoritative heap
  copy or whole-database materialization.
- Replace Relational's independent segmented-filesystem persistence and
  recovery path. Remove its obsolete configuration, implementation and usage
  guidance at cutover.
- Preserve explicit memory-only execution without treating it as durable
  evidence.

Complete when an ordinary Relational transaction survives process death and
fresh reconstruction with exact records, history and identities. A crash after
durable commitment but before live installation must not lose the commit, and
new allocations must not reuse recovered identities incorrectly.

## Milestone 3: Production host and original-result retry

A file-backed application survives lost responses and returns the original
committed result after restart.

Requirements:

- Open and recover the durable application through the production Query host,
  using one Store instance and the existing semantic owners.
- Persist World product identity, branch head, application occurrence and
  canonical history together with the associations needed to resolve the
  original Query idempotency result.
- Recover any pending settlement or unpublished owner custody created by the
  supported operation. Preserve existing partial and indeterminate outcomes.
- Preserve the difference between an owner effect and a completed application
  publication. Recover an interrupted association without losing the effect
  or inventing application success.
- Reject unsupported operation combinations before their first effect.
  Refuse opening an existing home whose required state cannot be recovered.
- Preserve application build identity, program admission and the host's real
  execution authority.
- Make durable acknowledgment independent of orderly close. Prevent competing
  writable opens and stale handles from authorizing new writes.
- Replace superseded application-open and checkpoint-transport entry paths as
  their callers cut over.

Complete when a real application performs a mutation with an idempotency key,
loses its response, is killed without closing, reopens in a fresh process and
returns the original occurrence and exact branch state on retry without
executing the mutation again.

Cover interruption before commitment, after durable publication, between owner
effects and product movement, and before response delivery. An attempt with
proven no effect may execute once on retry; a completed attempt returns its
original result; an unresolved effect retains its recovery posture. Changed
intent or foreign affinity must not reuse the original result.

## Milestone 4: Database scale, branches and sustained operation

The durable database supports branching, historical access and repeated use
with data larger than memory.

Requirements:

- Persist branch creation, fork, advancement, archive and deletion, including
  retired names and explicit ancestry.
- Share immutable base data across forks; storage and update work follow
  changes rather than copying the entire branch.
- Preserve retained historical reads, lineage and schema boundaries through
  restart.
- Complete cold-read and working-set support. Active native data is admitted
  in chunks; historical data and cold reads do not require full materialization.
  Opening a branch does not pin its entire contents.
- Preserve existing issued/context partition identity and placement semantics.
- Support progress on independent branches without a whole-Store preparation
  lock.
- Join retention and reclamation with live readers, shared roots, checkpoints
  and pending recovery obligations. Reused physical addresses cannot be
  mistaken for earlier allocations.
- Retain enough canonical coverage to reconstruct current truth and every
  promised historical view. Physical accelerators cannot become the sole
  surviving authority.

Complete when larger-than-memory reads and local edits remain local as total
data and retained history grow, and repeated fork/edit/drop/crash/reopen cycles
reclaim eligible space without changing retained observations. Evaluate
meaningful growth behavior rather than fixture-specific exact counts.

## Milestone 5: Complete runtime recovery

The supported application runtime resumes its authored state, history and
mandatory obligations through the same durable path.

Requirements:

- Persist and reconstruct Signal authored state and mixed Relational/Signal
  publications through their existing owners.
- Complete durable World history, identity issuers, pending settlements and
  unpublished owner recovery.
- Preserve program revisions and support lifecycle, idempotency results,
  workflows, dispatch outbox/correlation, inbound receipts and external
  completion custody.
- Recover in-flight obligations after crashes even when an orderly close
  would have refused. Reacquire process-local resources without restoring
  expired handles as authority.
- Preserve 9.18 correction occurrences, alternatives, causal links, retained
  original meaning and original-owner material needed for continued editing.
- Support dependency edits and original-owner advancement after reopen,
  including required-output settlement.
- On application build changes, preserve authored source and references while
  invalidating incompatible derived outputs. Regeneration must use the normal
  demand and settlement owners.
- Resolve interrupted external work without repeating completed effects or
  claiming that an unknown remote effect was undone.
- Remove remaining competing durable recovery paths and correct public
  capability claims and usage documentation.

Complete when production journeys resume mixed-owner work, pending settlement,
a workflow, ambiguous external completion and correction on a divergent branch
after process death. Destroying discardable derived state must not lose
authored truth or mandatory custody.

This closes the runtime database cutover. Advanced replication, backup/PITR,
security lifecycle, bulk ingest and broader platform qualification remain
separate work; their owners and return triggers are recorded in
[deferred work](../deferred-work.md#worth-store).
