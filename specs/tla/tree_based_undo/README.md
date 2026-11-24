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

## Hand traces

The DST harness writes generated traces as `seed-<n>.ndjson`. These stay
ignored; only `hand-*.ndjson` is versioned.

| Trace | Expected | Contents |
| --- | --- | --- |
| `hand-accepted` | exit 0 | 15 steps: create, coupled create, refused partial retirement, cascade correction, undo across it, branch after undo, checkout to the retired sibling, correction replay, stop, record, maintain, out-of-band write, diverged refusal, close. |
| `hand-co-retire` | exit 0 | Replays a recorded correction on its line after its source was reversed. |
| `hand-writer-out-of-band` | exit 0 | A stop at Recording; `Begin(edit)` then `Commit`; `Begin(gesture)`; an out-of-band retire under the writer; `Discard`; maintenance; close. |
| `hand-branch-redelete` | exit 0 | Create, delete by correction, undo, sibling edit, delete on the sibling, then a checkout back to the first delete. There a fresh-key delete on the same source is refused, because the reversal fact follows the selected line. |
| `hand-replay-other-source` | exit 0 | A correction key reused on another source step with the same derived input drifts; an edit under that key drifts; the key on its own source replays; a fresh key on the other source records. |
| `hand-writer-key-envelope` | exit 0 | A writer commits under `k2`. An envelope under `k2` then records a new step, because the namespaces are separate (refinement (v)). Its replay answers `AlreadyRecorded` with the envelope's own step. The flat pre-(v) journal rejects this trace (exit 11, pc = 4). |
| `hand-rejected-fresh-generation` | exit 11, pc = 5 | `hand-accepted`, but the undo across the retirement reports a fresh generation. That is what re-running `restore_retired` instead of navigating would do. |
| `hand-rejected-writer-hides-divergence` | exit 11, pc = 7 | `hand-writer-out-of-band`, claiming the head is current after the discard. |
| `hand-rejected-readable-omits-divergence` | exit 11, pc = 2 | `hand-accepted` with `postHeadDiverged` left out of a record where the document is readable. |
| `hand-rejected-restore-loses-fact` | exit 11, pc = 6 | `hand-branch-redelete`, but the checkout back to the first delete reports no active reversal fact. |
| `hand-rejected-other-source-replays` | exit 11, pc = 5 | `hand-replay-other-source` with today's defect: the reused key on the other source answers `AlreadyRecorded`. |
| `hand-rejected-envelope-replays-writer-key` | exit 11, pc = 4 | `hand-writer-key-envelope`, but the envelope under the writer's key replays the writer's step, as the flat pre-(v) journal said (DST seeds 0x7b020035 and 0x7b020108). |
