# Correction cancellation witness proofs (public #388)

`Cancellation.lean` models `require` in
`restored_support/cancellation.rs`: the capture loop, the pairing loop and the
abstract `require_pair` relation. A Memory Home takes each performed witness
from its session store and a Directory Home restores it from durable custody;
`require` reads only the witness map, so equal witnesses give the same answer.
Court 5 (`durable_witness.rs`) shows a restored witness equals the captured one.

## Run

```
python tools/lean/run_lean.py check specs/lean/correction_witness/Cancellation.lean
```

Expected: `Lean kernel audit PASSED`. No `sorry` and no axiom outside
`propext`, `Classical.choice` and `Quot.sound`.

## Theorems

| Theorem | Property |
|---|---|
| `capture_congr`, `pairs_congr` | each loop reads only the witnesses of selected occurrences |
| `equal_witnesses_equal_answer` | equal witness maps give the same answer (Directory = Memory) |
| `accepted_witnessed` | an accepted correction had a witness for every occurrence |
| `restricted_answer` | a restore that loses witnesses answers `unavailable` or as before |
| `absent_never_accepts` | losing witnesses never turns a refusal into an acceptance |
| `absent_only_unavailable` | losing witnesses turns an acceptance only into `unavailable` |

The `decide` examples pin the classification: a cancelled pair accepts, a bad
pair mismatches, a reapplication and an uncovered occurrence diverge, and a
missing witness is unavailable.

## Drift guard

The header's `-- proof-mirror:` lines record the Rust ranges (`require` and
`require_pair`). `python specs/proof_mirror_check.py` reports any range whose
source changed; after reviewing the proofs against the new code, refresh them.
