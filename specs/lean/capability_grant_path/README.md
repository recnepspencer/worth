# Standing grant path proofs (W9A-2)

`GrantPath.lean` models one candidate capability grant path as lowered by
`compile_grant_path` (`capability_lowering.rs`) and `extent_status::lower`:
grant-level predicates, the workflow equality a workflow-tenure grant carries,
and one equality predicate per `require_active` binding on every path ordinal
from the head (or context root) onward whose kind is the binding's. Relational
observation admits a path only when every predicate holds, and validity
compares the grant's inclusive bounds with one sample of the admission instant.

## Run

```
python3 tools/lean/run_lean.py check specs/lean/capability_grant_path/GrantPath.lean
```

Expected: `Lean kernel audit PASSED`. No `sorry`, no warnings and no axiom
outside `propext`, `Classical.choice` and `Quot.sound`.

## Theorems

| Theorem | Property |
|---|---|
| `extent_narrows` | declaring extent liveness only removes admissions |
| `inactive_root_revokes` | an extent entity on the path holding another status refuses every grant, principal and tenure: rooted revocation needs no grant row change |
| `standing_admits_iff` | a standing grant admits exactly on its base predicates and extent liveness |
| `workflow_admits_standing` | a standing grant is a workflow grant minus the workflow equality |
| `expired_stays_expired` | on a UTC (or epoch) timeline, once past `not_after` a grant never admits again |
| `expired_stays_expired_local` | on the local-date timeline the same holds exactly when the calendar's date is monotone |

The final `example`s pin the lowering: an active extent admits, an inactive
one refuses, a workflow mismatch refuses, the grant itself (ordinal 1) is not
an extent ordinal, and a non-monotone calendar (a fall-back across midnight)
can make a past local date current again. That last case is civil-calendar
meaning, not a platform defect: no admission caches local-date expiry.

## Drift guard

The header's `-- proof-mirror:` lines record the lowered Rust ranges.
`python3 specs/proof_mirror_check.py` reports any range whose source changed;
after reviewing the proofs against the new code, refresh the digests.
