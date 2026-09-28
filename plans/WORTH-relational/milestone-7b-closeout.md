# Milestone 7B Closeout and 7C Handoff

## Purpose

This document closes Milestone 7B and defines the exact authority boundary
that Milestone 7C must consume.

7B is complete when merge planning is no longer advisory.

It is not execution, but it is already authority.

7C must therefore treat 7B artifacts as canonical semantic inputs, not as
helpful hints to reinterpret.

## What 7B Now Guarantees

The runtime now emits one canonical merge-planning artifact:

- `MergePlanningArtifactCore`

That artifact now carries:

- request truth
- schema semantic snapshot truth
- execution authority contract
- merge-base truth
- ancestry truth
- identity discovery truth
- conflict classification truth
- causal annotation truth
- policy resolution truth
- lowered execution-readiness truth
- decision log truth
- digest basis truth

The most important closeout change for 7C is this:

- every lowered record now carries exactly one canonical record decision

That decision is:

- `LoweredRecordDecision::Execute(...)`
- `LoweredRecordDecision::Block(...)`
- `LoweredRecordDecision::Reject(...)`

7C should consume that enum directly.

It must not reconstruct execution meaning from the former optional shape of:

- execution bundle present or absent
- denial bundle present or absent
- blocked reason fields
- rejected reason fields

Those fields still exist as summaries and diagnostics, but the canonical record
authority surface is now the record decision enum.

## 7C Consumption Contract

The artifact now carries an explicit `MergeExecutionAuthorityContract`.

Its current meaning is:

- decision surface: `LoweredRecordDecisionOnly`
- identity authority: `ConsumeCanonicalLoweredArtifactOnly`
- conflict authority: `ConsumeCanonicalLoweredArtifactOnly`
- policy authority: `ConsumeCanonicalLoweredArtifactOnly`
- value authorization: `MustNotWidenBeyondAuthorizedAspectValueSurface`

This means 7C is not allowed to:

- re-run identity matching
- re-run conflict classification
- re-run policy resolution
- widen value authority beyond what 7B already authorized
- reinterpret blocked/rejected records as executable

This means 7C is allowed to:

- execute admitted record decisions
- materialize denied outcomes as explicit refusal/no-op execution results
- consume authorized aspect value surfaces exactly as granted
- fail closed if required execution inputs are absent

## Three-Way Base Authorization Contract

7B now treats three-way merge as a real future execution obligation, not a
future comment.

The base-value authorization rule is now explicit.

Current contract:

- `SourceOnlyAddition`
  - source: `ConsumeVisibleValue`
  - target: `NotAuthorized`
  - base: `NotAuthorized`

- `ExactSharedTruth`
  - source: `EqualityWitnessOnly`
  - target: `EqualityWitnessOnly`
  - base: `NotAuthorized`

- `SchemaDeclaredCorrespondence` / reconciliation path
  - source: `ConsumeVisibleValue`
  - target: `ConsumeVisibleValue`
  - base: `ConsumeBaseValue`

This is the important meaning:

- the reconciliation path is now explicitly authorized to inspect base truth
  during 7C execution
- source-only adoption and exact shared preservation are not authorized to
  consult base as an execution input today

If 7C needs more base authorization than this, the correct move is to extend
7B authority first, not to improvise inside execution.

## Unsupported Semantics That Are Now Rejected Up Front

To keep the codebase honest, unsupported merge semantics are no longer allowed
to enter planning and silently degrade.

Schema registration now rejects:

- merge policies:
  - `LastWriterWins`
  - `MonotonicCounter`
  - `AdditiveSet`
  - `Custom(_)`

- identity bases:
  - `StructuralFingerprint`
  - `Custom(_)`

That means the active 7B planning surface currently supports:

- merge policies:
  - `FailOnConflict`
  - `PreferRicher`

- identity bases:
  - `StorageIdentity`
  - `LineageIdentity`
  - `DeclaredKeySet(...)`

This is intentional.

It is better for 7C to inherit a smaller truthful surface than a larger
dishonest one.

## What 7C Must Treat As Already Proven

7C may assume the following are already settled by authority:

- branch direction and merge intent
- merge-base selection rule and result
- request-scoped schema semantic snapshot
- request-scoped identity basis set
- candidate correspondence and validated schema correspondence
- conflict class and relation conflict evidence
- causal disposition
- per-aspect policy resolution
- per-aspect authorized value surface
- per-record admitted/blocked/rejected decision kind

7C should not add second implementations of those concerns.

## What 7C Still Owns

7C still has a hard job, but it is narrower now.

Its real job is:

- consume `LoweredRecordDecision`
- materialize authoritative commit-intent from admitted record decisions
- preserve denied record outcomes as explicit non-execution truth
- commit canonical merge execution artifacts
- ensure CDC, durability, replay, and diagnostics consume execution truth
  without semantic drift

Future semantic expansions that belong after 7B:

- `LastWriterWins`
- `MonotonicCounter`
- `AdditiveSet`
- structural-fingerprint identity
- custom identity bases
- custom merge policies

Those should only land when they have full execution semantics, diagnostics,
digest participation, and replay/durability parity.

## Recommended First Steps For 7C

1. Introduce the canonical merge execution artifact that consumes
   `LoweredRecordDecision`.

2. Implement admitted execution only, but preserve blocked and rejected
   decisions as explicit execution-denial outputs.

3. Keep execution phase types sealed so no caller can construct execution
   meaning from weaker planning surfaces.

4. Start with the currently supported truthful surface:
   - `FailOnConflict`
   - `PreferRicher`
   - storage identity
   - lineage identity
   - declared-key correspondence

5. Treat any need for unsupported semantics as a stop-and-extend boundary, not
   as permission to improvise host logic.

## Final Rule

If 7C finds itself needing to answer the question:

- "what does this merge mean?"

then 7C is already doing too much.

7B's closeout goal was to ensure that 7C only needs to answer:

- "how do I execute the already-authorized merge decision canonically?"
