# Relational retained panic proof audit

The inline `Proof:` comments beside production sites are the only maintained per-site proofs. The former copied file:line table was removed because it could drift and its ambient invariants were not sufficient evidence.

Regenerate the production inventory from the reviewed checkout with:

```text
python plans/WORTH-relational/cutover_panic_census.py target/lane-evidence/review-sites-after.json
```

The script requires Python `tree_sitter` and `tree_sitter_rust`. The lane wrapper records the exact command, starting HEAD, dirty status, times, exit code and counts. The generated JSON includes each complete source expression and its inline proof, including assertion proofs. It is evidence for that exact checkout, not a second maintained proof source.

Retained sites are confined to concrete construction or local control-flow proofs: immutable closed native encodings already admitted by artifact construction; fixed-size native hash tuples; private sorted map/column/tree algorithms; immediate Some/variant checks under the same exclusive borrow or mutex; and non-cloneable obligations, reservations and performed witnesses whose constructors install the state that consuming cleanup removes. Runtime registry and allocator proofs name their checked issuers and exclusive guards. Media readmission does not directly populate those private bookkeeping states.

A Full demand is not a proof for a later infallible cold lookup. Neither eager commit indexes nor a plausible id/owner invariant prove a payload is resident or a successor exists. Those cases now return typed refusals or carry the actual admitted value. The public cold-read matrix covers all three StorageAccess adjacency doors for every existing region/history/symbol failure class, and a cold/warm court checks their exact relation answers.
