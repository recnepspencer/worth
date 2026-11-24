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
python tools/tla/run_tlc.py check $M/TreeBasedUndo.tla 8    # full check: -Xmx4g, about 26 min
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
| `hand-rejected-fresh-generation` | exit 11, pc = 5 | `hand-accepted`, but the undo across the retirement reports a fresh generation. That is what re-running `restore_retired` instead of navigating would do. |
| `hand-rejected-writer-hides-divergence` | exit 11, pc = 7 | `hand-writer-out-of-band`, claiming the head is current after the discard. |
| `hand-rejected-readable-omits-divergence` | exit 11, pc = 2 | `hand-accepted` with `postHeadDiverged` left out of a record where the document is readable. |
| `hand-rejected-restore-loses-fact` | exit 11, pc = 6 | `hand-branch-redelete`, but the checkout back to the first delete reports no active reversal fact. |
| `hand-rejected-other-source-replays` | exit 11, pc = 5 | `hand-replay-other-source` with today's defect: the reused key on the other source answers `AlreadyRecorded`. |
