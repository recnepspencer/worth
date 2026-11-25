# Authored index coverage proofs (public #384)

`Coverage.lean` proves, unbounded, the properties the TLA+ model in
[`specs/tla/authored_index_coverage`](../../tla/authored_index_coverage/README.md)
checks on small bounds: a commit's persisted bound certifies only the receiving
catalog's coverage before it, legacy, unavailable, mismatched and
stale-incarnation packets certify nothing, and every checkpoint and WAL-tail
split admits exactly what WAL replay admits.

## Run

```
python tools/lean/run_lean.py check specs/lean/authored_index_coverage/Coverage.lean
```

Expected: `Lean kernel audit PASSED (18 theorem(s)).` No `sorry` and no axiom
outside `propext`, `Classical.choice` and `Quot.sound`.

## Theorems

| Theorem | Property |
|---|---|
| `epoch_mono_of_ordered_prefix` | an ordered prefix readmits definitions with increasing epochs |
| `coverageBefore_regN_stable` | later registrations leave earlier coverage unchanged |
| `never_covers_later` | certified coverage never extends an index at or after the bound |
| `certify_some` | an honest bound within the catalog certifies its coverage |
| `reopen_eq_replay` | certification after a reopen equals replay minting |
| `legacy_refuses` | a legacy packet certifies nothing |
| `mismatch_refuses` | a bound that disagrees with the delta certifies nothing |
| `stale_incarnation_not_ext` | coverage from a replaced catalog never extends the new incarnation |
| `admit_compression_sound` | the admission run compression is sound |
| `admit_sound` | an admitted span holds every write of the index |
| `legacy_in_span_refuses` | a span over a legacy packet refuses |
| `stale_incarnation_refuses` | admission refuses stale-incarnation coverage |
| `admit_split_eq_replay` | every checkpoint and WAL-tail split admits as replay does |

The five `mutant_*` theorems show each guarded check matters: certifying from
the reopen-time next id, accepting legacy, re-minting on replay, readmitting out
of order, and dropping the run start each break the matching property.

## Drift guard

The header's `-- proof-mirror:` lines record the Rust ranges each definition
mirrors. `python specs/proof_mirror_check.py` reports any range whose source
changed; after reviewing the proofs against the new code, refresh the ranges.
