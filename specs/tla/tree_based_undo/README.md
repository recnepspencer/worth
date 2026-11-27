# Tree-based undo: TLA+ model and traces

`TreeBasedUndo.tla` models document history as built in Phase 1 plus the
Phase 2 corrections (issue #56). `TreeBasedUndoTrace.tla` checks one NDJSON
trace against it. Its header documents the record fields. A trace passes
only if each record is a transition the model allows, with the recorded
outcome and post-state.

## Running

Run from the repository root with `tools/tla/run_tlc.py`. It finds `java` and
`tla2tools.jar` (set `TLA_JAVA` / `TLA2TOOLS_JAR` to override), and it picks
the `.cfg` with the same name as the `.tla`. For `trace` it turns the NDJSON
into `TraceData.tla` (`Trace == << ... >>`, where `{"__set__": [...]}` is a
set) and runs TLC on the trace module with one worker.

```sh
M=specs/tla/tree_based_undo
python tools/tla/run_tlc.py check $M/TreeBasedUndo.tla 8    # full check: -Xmx4g, 20-26 min
python tools/tla/run_tlc.py check $M/TreeBasedUndoWedgeBug.tla 4
python tools/tla/run_tlc.py check $M/TreeBasedUndoFactBug.tla 4
python tools/tla/run_tlc.py trace $M/TreeBasedUndoTrace.tla $M/traces/<trace>.ndjson
```

Without the runner, a model check is
`java -cp tla2tools.jar tlc2.TLC -workers 8 -config TreeBasedUndo.cfg TreeBasedUndo.tla`.

TLC's exit codes are:
- 0: no error;
- 11: deadlock, which for a trace means a record the model does not allow;
- 12: an invariant was violated.

## Models

| Model | Expected | What it shows |
| --- | --- | --- |
| `TreeBasedUndo` | exit 0 (21,378,307 distinct states, depth 13) | Every invariant and property holds. |
| `TreeBasedUndoWedgeBug` | exit 12, `NoWedge`, 3 states | The writer-custody wedge the model rules out is reachable once re-introduced. |
| `TreeBasedUndoFactBug` | exit 12, `FactsFollowLine`, 4 states | Reversal facts that ignore navigation break `FactsFollowLine`. |

## What the Rust DST settled

- A request can stop at `Admission`, before its candidate forks. That parks
  nothing and changes nothing (seed `0x7b020000`: a stop at the history
  read's admission, then maintenance finds no pending release).
- Close retires the document branch, so a closed document's records may
  leave out `postLive`, `postGen`, `postVal`, `postOutA` and `postOutB`. A
  record of a document that is not closed must carry them.
- Maintenance also releases past heads, not only parked candidates. A move
  forks a new branch; when the document leaves that fork, by recording a
  step or by moving again, the fork is queued for `maintain_history`
  (`forked`, `heads`, `MaintainHead`; S1 probe: edit, undo, edit, then
  maintenance releases one). A move from the step's own branch queues
  nothing, and close drains the queue. `MaxHeads` bounds the queue for TLC
  as the platform's release capacity does; the trace configuration's 20
  covers the DST's 16 choices per seed.

- A write outside the document settles its program outputs in the same
  step (`OutOfBandMutation`). The model has no output wave, marks or
  readmission, so a settle that never ends is outside it, and no liveness
  property here could see one. `NoWedge` is about writer custody and holds
  either way. The DST checks settle progress instead: each call has a
  30 s deadline (`fixture/reads.rs`), and a spin refuses
  `DeadlineExceeded` as a disagreement. Seed `0x7b020039` found one (public
  #353): record, undo, record again (a fork), then write outside. The
  restored outputs' marks did not answer at the forked head, and each
  verified readmission joined the adopted row without marking it, so the
  required wave spun until the deadline. `dst::regressions` pins it.
- A correction step is not itself a correction source (coordinator ruling
  2026-11-25, option 1, under the spec owner's Phase 2 ruling Q2 that
  reapply is out of scope). The corrections sweep (seed `0x7b020007`) found
  the r2 model letting a correction reverse an earlier correction's
  retirement. The platform refuses to select a correction commit,
  `SourceNotCorrectable`, because a corrective binding declares no
  correction of its own. `MeaningDerivable` now excludes `CorrectionSteps`.
  Every other use of `MeaningDerivable` is a guard, and on reachable states
  `FactsAgreeWithSnapshots` reads the same as before. So the change only
  removes behaviours: the earlier full check's result carries over, and
  `WedgeBug`/`FactBug` and every hand trace were rerun (logs
  `ruling1-*.log`).
- Refinement (p) narrowed to the declared contract (ruling 2026-11-25,
  confirmed by the spec owner): a cascade co-retires only what the same
  source stage (binding) created, which is what `co_retire_created`
  declares. Every stage here creates one entity, so `CoRetired` is `FALSE`,
  and reversing `e1`'s creation while a same-step sibling from another stage
  lives refuses `StageRefused`. A cross-stage cascade waits on public #369,
  which is not a CAD Phase 3 prerequisite. The change only turns that
  recording into a refusal, so it removes behaviours, and the earlier full
  check's result carries over (corrections sweep, seed `0x7b020010`).
- Refinement (w), approved 2026-11-25 and narrowed by the 128-seed sweep
  (seed `0x7b02007f`): selecting an uncorrectable source refuses before any
  document call, so even on a diverged head. `Correctable(src, e)` is the
  part of `MeaningDerivable` that selection decides: the source stage
  created or retired `e`, and its step is not a correction. The document's
  head read comes next (`HeadDiverged`), and only then the derivation and the
  selected-line check, so a correctable source off the selected line refuses
  `HeadDiverged` on a diverged head. Refusal-only; no state.

## Hand traces

The DST harness writes generated traces as `seed-<n>.ndjson`. These stay
ignored; `hand-*.ndjson` and the DST baselines below are versioned.

| Trace | Expected | Contents |
| --- | --- | --- |
| `hand-accepted` | exit 0 | 15 steps: create, coupled create, refused partial retirement, cascade correction, undo across it, branch after undo, checkout to the retired sibling, correction replay, stop, record, maintain, out-of-band write, diverged refusal, close. |
| `hand-co-retire-across-stages-refused` | exit 0 | One step creates `e1` and `e2` (two stages); reversing `e1`'s creation refuses `StageRefused`: a cascade co-retires only what the same source stage created, so the live sibling makes it a partial retirement (refinement (p), narrowed). |
| `hand-rejected-co-retire-across-stages` | exit 11, pc = 2 | The former `hand-co-retire`: the reversal co-retires the sibling another stage created and records, as the r2 model's (p) allowed. |
| `hand-writer-out-of-band` | exit 0 | A stop at Recording; `Begin(edit)` then `Commit`; `Begin(gesture)`; an out-of-band retire under the writer; `Discard`; maintenance; close. |
| `hand-branch-redelete` | exit 0 | Create, delete by correction, undo, sibling edit, delete on the sibling, then a checkout back to the first delete. There a fresh-key delete on the same source is refused, because the reversal fact follows the selected line. |
| `hand-replay-other-source` | exit 0 | A correction key reused on another source step with the same derived input drifts; an edit under that key drifts; the key on its own source replays; a fresh key on the other source records. |
| `hand-writer-key-envelope` | exit 0 | A writer commits under `k2`. An envelope under `k2` then records a new step, because the namespaces are separate (refinement (v)). Its replay answers `AlreadyRecorded` with the envelope's own step. The flat pre-(v) journal rejects this trace (exit 11, pc = 4). |
| `hand-stage-distinct-corrections` | exit 0 | One step creates `e1` and `e2`. Key `k2` reverses `e2`'s creation, `k2` on `e1`'s creation of the same step drifts, and `k2` on `e2` again replays step 2. The model's `CorrectionIntent` already names the entity (the stage); the platform's meaning names the commit member (public #368). |
| `hand-correction-step-not-a-source` | exit 0 | Create `e1`, delete it by correction, then a fresh-key correction of that correction step refuses `SourceRefused` and changes nothing: a correction step is not a correction source. |
| `hand-diverged-head-underivable-source` | exit 0 | Create `e1`, write `e2` outside the document (the head diverges), then a correction of step 1's `e2` stage, which has no corrective binding: the head pass refuses `SourceRefused` before any head read (refinement (w)). The model without (w) rejects it (exit 11, pc = 3). |
| `hand-diverged-head-off-line-source` | exit 0 | Create `e1`, undo (step 1 leaves the line), write outside the document (the head diverges), then correct step 1's creation: `HeadDiverged`, because the selected-line check follows the head read. |
| `hand-rejected-off-line-source-before-head-read` | exit 11, pc = 4 | The same, answering `SourceRefused`, as (w) before its narrowing allowed. |
| `hand-rejected-correction-step-corrected` | exit 11, pc = 3 | `hand-correction-step-not-a-source`, but the correction of the correction records and restores `e1`, as the r2 model allowed. |
| `hand-rejected-fresh-generation` | exit 11, pc = 5 | `hand-accepted`, but the undo across the retirement reports a fresh generation. That is what re-running `restore_retired` instead of navigating would do. |
| `hand-rejected-writer-hides-divergence` | exit 11, pc = 7 | `hand-writer-out-of-band`, claiming the head is current after the discard. |
| `hand-rejected-readable-omits-divergence` | exit 11, pc = 2 | `hand-accepted` with `postHeadDiverged` left out of a record where the document is readable. |
| `hand-rejected-restore-loses-fact` | exit 11, pc = 6 | `hand-branch-redelete`, but the checkout back to the first delete reports no active reversal fact. |
| `hand-rejected-other-source-replays` | exit 11, pc = 5 | `hand-replay-other-source` with today's defect: the reused key on the other source answers `AlreadyRecorded`. |
| `hand-rejected-envelope-replays-writer-key` | exit 11, pc = 4 | `hand-writer-key-envelope`, but the envelope under the writer's key replays the writer's step, as the flat pre-(v) journal said (DST seeds 0x7b020035 and 0x7b020108). |
| `hand-rejected-stage-correction-replays-other-stage` | exit 11, pc = 3 | `hand-stage-distinct-corrections`, but `k2` on `e1`'s creation replays step 2: one meaning for two stages of one step. |

## DST baselines

Three owned-outputs DST traces recorded on public master `f5e92d0d36`
(`the_document_agrees_with_the_oracle`, default 16 operations, no
corrections), copied unchanged from `seed-<n>.ndjson`. Held seeds
(`0x7b02000f`, public #429) are excluded. Tree-Based Undo Phase 2E
regenerates them and compares them byte for byte; a difference is explained
or treated as a regression.

| Trace | Expected | Contents |
| --- | --- | --- |
| `dst-0x7b020000` | exit 0 | A stopped edit, a writer step, an edit, undo, a refused redo, a fork edit, a checkout back to the first branch, an out-of-band write, then edits and a checkout refused `HeadDiverged`, two maintenance passes, close. |
| `dst-0x7b020019` | exit 0 | Envelope refusals (undeclared operation, empty, intent drift, stop, stage refusal), a checkout to the root and a fork there, a second fork after undo, a refused redo, a checkout, close. |
| `dst-0x7b02002d` | exit 0 | A fork at the root (checkout to node 0, then a new edit), a checkout back, `redo_to` choosing the second child, undo, discarded writers, close. |

## Proofs of the implementation kernels

The model checks the protocol. Two kernels of the Phase 2 implementation are also proved against their source:
- the journal's O(log) jump-pointer lift (`specs/verus/tree_based_undo/`, Verus, plus `specs/lean/tree_based_undo/JumpLift.lean`);
- restored support's navigation supersession rule (`specs/lean/tree_based_undo/Supersession.lean`).

Their READMEs give the commands. `python specs/proof_mirror_check.py` fails when any source range a proof mirrors has changed.
