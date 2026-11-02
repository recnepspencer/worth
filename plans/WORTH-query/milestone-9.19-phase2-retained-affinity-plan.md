# 9.19 Phase 2: retained Source affinity without live-cache custody

## Boundary brief

The current Query registry owns Source branch cells until disposal. Native roots
retain an exact ManagedSourceRootImage plus a weak Core link; observations retain
Core strongly. Core.current owns the newest branch image. Thus removing a closed
branch lookup can break genuine retained reads, while an old Source observation
can keep an unrelated newer cache alive. World ReuseExact aliases and admitted
Native bases retain roots as Observation obligations, which do not block Native
branch deletion as active operations. These are distinct from active transactions.

Native root attachment and actual fork preparation are the only constructors of
Source/root custody. Every public observation, eviction and rebuild resolves an
actual snapshot and current companion registration first. Query separately owns
READ, principal, World currentness and purpose admission. Numeric root/branch or
registration equality cannot construct that custody. Transaction conditions must
still consume an actual live cell and exact reserved image.

A private Native runtime affinity object may retain immutable runtime/branch/
registration metadata and allocation custody, but never Core.current or a request.
The actual core mints this once before sealing; root attachments and observations
share that exact object. A weak Core link retains live-current checks only. The
selected image remains the canonical cache authority. No new application-facing
API, proof issuer, fallback root reconstruction or public currentness is introduced.

The next cleanup slice must consume actual Native deletion custody after owner
locks and remove Query's exact lookup. This prerequisite does not claim that
notification/removal or complete branch-close reclamation is already implemented.

## Implementation plan

1. Add a private funded ManagedSourceCellAffinity in the existing Native cell
   owner. Its only constructor derives metadata from the actual cell. Reserve
   original retained memory before its Arc and branch string exist. Mint once
   before compound sealing and genuine fork attachment, with no work at cutover.
2. Attach that affinity to the exact Native root image. Source observations retain
   it and their scope payload while holding Core weakly. Centralize exact selected
   affinity/image admission for observe, evict and rebuild. Preserve registration,
   runtime, branch, root, commit and position checks without requiring live Core.
3. Keep live-current and transaction precondition checks dependent on upgrading
   the exact weak Core; an expired cell cannot authorize a current write. Forks
   retain their existing live-source validation and do not gain a recovery door.
4. Add a narrow Native protocol court using real publication, a genuine fork and
   snapshot, real deletion, and release of the participant's live lookup. Prove
   retained read/evict/refill custody survives, an observation does not retain the
   live Core, wrong registration/runtime/branch refuses, and a write condition
   cannot borrow expired live custody. This court claims cache protocol only,
   not projected-row or invariant READ correctness.
5. Run focused Native managed-cache owner tests, then existing public managed-index
   controls for the Query handoff. Run mandatory boundary/context, format, dirty
   caps/function advisories and independent QA; update qualification precisely and
   commit. Use the shared target, no concurrent Cargo or build variants.

## Remaining scope

Query branch retirement notification/removal follows this prerequisite. Recovery,
physical suspension/rematerialization, standalone resource cutover, hostile
lifecycle controls, growth qualification and later milestone phases remain open.

## Evidence and next boundary

The Native library check passes. The new real-deletion court passes, as does the
six-case managed-cache owner group. All 25 public managed-index controls pass,
including the allocation journeys. Dirty formatting and line caps pass for nine
Rust files; the function scan reports ten advisories and no scan errors. The
independent reviewer accepts the authority signatures and one coherent sequential
owner journey, with no extraction request. Final boundary-check and agent-context commands both exit successfully. The
existing Native fork refusal/success teardown control also passes. All 25 public
managed-index controls and six Native cache controls pass after the fork-grant
correction. Independent QA gives unconditional commit clearance; automatic Query
branch-close reclamation remains explicitly unqualified.

The qualifier deliberately covers retained Native affinity after the participant
relinquishes its live lookup. Automatic Query lookup retirement is still required.
The lawful trigger is a successful Native DeletedRelationalBranch receipt, not
World occurrence retirement, a branch name, WaitingForActiveOperations or an
unresolved durable fate. ReuseExact creates no Native component custody to delete.
The current product owner-cleanup component port returns after the Native owner
mutex, but application close still holds command/application coordination guards
through cleanup.retry. A new follower cannot simply remove and finally destroy
opaque payload there. The next boundary review must follow close, retry and
recovery callers and carry final retirement beyond all their guards, preserving
completed effects across original-request final polling. This is required work,
not a reason to weaken the retained-image or destruction contract.

The ownership review found one adjacent funding gap: a weak Core still retains
its Arc allocation after Core value destruction. The affinity therefore shares
Core's existing aggregate fork reservation, without a second charge or a backedge.
The court keeps a refilled observation beyond all runtime/root/cell owners and
checks the actual fork reservation remains live until that observation closes,
then requires final zero. Observation/root field order releases Weak<Core> before
the last affinity/grant share. This strengthens lifetime evidence without adding
another test family or widening the generic nonfork cell allocation protocol.
