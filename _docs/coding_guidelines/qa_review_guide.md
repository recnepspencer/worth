# QA Review Guide

Use this guide while designing a specification and reviewing an implementation.
It provides shared review categories, not a ledger, checklist database, or
certification system.

Select the categories materially relevant to the change. Do not manufacture
concerns or tests merely to fill every category. Specifications should name the
important QA considerations in ordinary prose; code review decides whether the
implementation and tests address them adequately.

## Requirements and correctness

- Is the intended behavior explicit?
- Are invalid, denied, partial, and terminal outcomes defined where relevant?
- Could the implementation satisfy the wording while defeating the intent?
- Are consequential state changes observable?

## Architecture and authority

- Is there one honest owner for each decision and truth source?
- Does dependency direction preserve authority?
- Can a weaker representation, facade, fixture, or adapter bypass the owner?
- Does the change create a competing authority path?

## Security and privacy

- Can authority, identity, provenance, or permission be forged?
- Can information cross an unauthorized boundary?
- Are secrets, retained data, and diagnostics handled appropriately?
- Do relevant denial paths fail closed?

## Lifecycle, concurrency, and recovery

- Who owns creation, cancellation, settlement, cleanup, and disposal?
- What happens after interruption or partial effect?
- Are retries, duplicates, reordering, or competing operations relevant?
- Can resources or authority escape after replacement or shutdown?

## Persistence and compatibility

- What must remain true across restart, migration, or version coexistence?
- How are stale, malformed, partial, or corrupted states handled?
- Is rollback possible, and when does an effect become irreversible?
- Are compatibility promises explicit?

## Performance and resources

- What is the ordinary path and what work does it perform?
- Which dimensions scale?
- Are memory, queues, retained state, retries, and reconstruction bounded?
- Are expensive tests and setup proportionate to the risk?

## Integration and platform behavior

- Does the test reach the real boundary relevant to the claim?
- Are OS, protocol, adapter, timing, or external-system assumptions explicit?
- What environment is required?
- Is environment denial distinguished from a product defect?

## Public API, DX, and operability

- Is the intended caller path clear and compiler-supported?
- Are illegal uses difficult or impossible where that has product value?
- Are diagnostics actionable?
- Can operators identify failure, blocked posture, and recovery steps?

## Composition and maintainability

- Does each file and module own one predictable responsibility?
- Can expected successors be added without unrelated edits?
- Are facades honest?
- Are naming, placement, and deletion boundaries clear?

## Tests and evidence

- What plausible defect should each important test or test family catch?
- Is the world credible for the behavior under review?
- Is the observation independent enough to detect the defect?
- Can the test pass because setup or observation failed?
- Is the chosen boundary and cost appropriate?
- Does code review consider the evidence sufficient?

## Specification use

Add a short `QA considerations` section to a specification when the change has
material risks worth calling out. Name only the relevant categories and expected
evidence in prose. Do not create identifiers, status rows, predecessor chains,
or a machine-readable evidence registry.

Example:

> Architecture review must confirm that runtime remains the sole input-affinity
> owner. Lifecycle review must cover input arriving during replacement and
> shutdown. Focused tests should cover affinity and typed denial; the real
> Windows journey is required for release validation.

## Code-review use

Review the specification's QA considerations, apply any additional categories
made relevant by the actual implementation, inspect the tests, and state material
findings. A concise review summary may name the categories covered and explain
why other categories were not material.

Every delegated review prompt must state its reading boundary. For incremental
corrections, give the exact last-approved-to-candidate diff, changed-file and
finding inventory, invalidated seams, and prior accepted evidence. Instruct the
reviewer to read that patch once, then revisit only named hunks or line ranges.
Diff reads use at most five context lines; high-context or whole-file diff
output is prohibited. The reviewer must not reopen the whole diff or reread the
full milestone, crate, kernel, manifest set, or dirty surface. Any necessary
scope expansion is announced and justified before inspection.

Holistic reading is reserved for an explicitly named phase or milestone closure
gate. Even then, provide the commit inventory and completed evidence up front,
forbid whole-diff dumps and repeated file reads, and ask the reviewer to trace
the material semantic clusters rather than reconstructing the repository.

The review ends when the implementation is sound, the accepted tests pass, and
the reviewers judge the remaining risk acceptable. Do not add evidence whose
primary purpose is to certify other evidence.
