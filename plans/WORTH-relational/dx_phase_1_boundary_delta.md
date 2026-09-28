# WORTH Relational DX Phase 1 Boundary Delta

## Purpose

This is the live-code checkpoint for the start of Phase 1.

It answers two questions:

1. what boundary cleanup calls are already real in code
2. what fake seams or public-story leaks are still left

This is not a philosophy doc.

It is the "what is true right now in the codebase?" doc for the published
boundary.

This file is intentionally a start-of-phase snapshot.

It does not reflect later cleanup that closed the phase.

For the closeout state, use:

- [`dx_phase_1_review.md`](./dx_phase_1_review.md)
- [`dx_phase_2_review.md`](./dx_phase_2_review.md)

---

## Verdict

The first seam-cleanup pass stuck.

The previously accepted fake-seam removals are real in code.

No new cluster of equally bad runtime helper backdoors showed up in the follow
up audit.

The remaining Phase 1 problems are now more specific:

- `harness` is still a first-class facade module
- the promoted lanes still exist mostly as raw runtime seams instead of fully
  owned public lanes
- `RelationalRuntimeApi::runtime()` still reads like a second official setup
  door instead of a clearly subordinate shortcut

That is a much tighter problem than where we started.

---

## Confirmed Already-Implemented Boundary Cleanup

These removals are now real in the live code:

### Removed From Public Runtime Boundary

- `publication_authority()`
  is `pub(crate)`
- `storage_authority()`
  is `pub(crate)`
- `lineage_access()`
  is `pub(crate)`
- `lineage_authority()`
  is `pub(crate)`

### Removed Specialist Backdoor

- `MergeAccess::runtime()`
  is gone

### Promoted Into Exported Runtime Vocabulary

The runtime surface now exports these explicitly:

- `InvariantAccess`
- `SimulationAccess`
- `SimulationAuthority`
- [`VisibilityReadContext`](../../crates/worth-relational/src/visibility/materialization/read_records/mod.rs)
- [`VisibilityRetentionAuthority`](../../crates/worth-relational/src/visibility/retention/retention_authority.rs)

That means the code already reflects the first round of:

- remove fake seams
- acknowledge real lanes

Even though the naming and lane-ownership story is not finished yet.

---

## Follow-Up Audit Result

The follow-up code scan did not reveal another batch of boundary smells at the
same severity level as:

- `publication_authority`
- `storage_authority`
- `lineage_access`
- `lineage_authority`
- `MergeAccess::runtime`

That matters.

It means Step 2 is not secretly hiding a second wave of equally bad cleanup
calls.

The remaining issues are now mostly:

- public-story hierarchy
- lane ownership
- support-surface de-emphasis

Not:

- surprise runtime backdoors everywhere

---

## Remaining Phase 1 Smells

## 1. `facade::harness` Is Still Publicly Loud

Source:
[`facade.rs`](../../crates/worth-relational/src/facade.rs)

Current reality:

- `harness` is still a top-level facade module
- it still exports fixture and harness planning types directly from
  [`presentation/harness.rs`](../../crates/worth-relational/src/presentation/harness/mod.rs)

Why this still matters:

- it is the clearest remaining support-shaped public story leak
- it still competes for attention as if it were part of the normal product
  contract

Call:

- still unresolved for Phase 1
- should be handled in the dedicated `harness` cleanup step

## 2. Promoted Lanes Are Exported, But Not Fully Product-Owned Yet

Current reality:

These are still primarily discovered as runtime methods:

- `visibility_reads()`
- `retention_authority()`
- `simulation_access()`
- `simulation_authority()`
- `invariant_access()`

Why this still matters:

- they are real
- they deserve to stay public
- but they still feel like runtime side seams instead of clearly owned public
  lanes

Call:

- not a removal issue anymore
- now a lane-ownership and naming issue

## 3. `RelationalRuntimeApi::runtime()` Is Still Boundary-Ambiguous

Source:
[`presentation/api.rs`](../../crates/worth-relational/src/presentation/api.rs)

Current reality:

- `builder()` exists
- `runtime()` also exists and constructs a default runtime directly

Why this matters:

- it is not a fake seam
- but it does weaken the "one obvious setup door" story
- it still reads like a second equally respectable setup path

Call:

- not a Step 2 leak-removal issue
- definitely a Phase 1 hierarchy issue

---

## What Step 1 And Step 2 Actually Established

Phase 1 Step 1 and Step 2 are now clean enough to say this:

- the biggest fake seams have already been removed
- the live code still matches those removal decisions
- there is not a hidden second list of equally bad runtime backdoors waiting to
  surprise us
- the next phase-1 work should focus on:
  - lane ownership
  - setup hierarchy
  - `harness`

That is a good place to be.

It means Phase 1 can stop spending energy on generic seam hunting and start
making harder, more interesting boundary choices.

---

## Practical Next Move

The next high-value moves are:

1. decide the official owner and wording for:
   - read truth
   - validation
   - compiled artifacts
   - retention
2. decide what to do with `RelationalRuntimeApi::runtime()`
3. remove or contain `facade::harness` from the main public story

That is the real remaining Phase 1 frontier.
