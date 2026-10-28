# Relational retained panic proof audit

The inline `Proof:` comments beside production sites are the only maintained per-site proofs. The former copied file:line table was removed because it could drift and its ambient invariants were not sufficient evidence.

Regenerate the production inventory from the reviewed checkout with:

```text
python plans/WORTH-relational/cutover_panic_census.py target/lane-evidence/review-sites-after.json
```

The script requires Python `tree_sitter` and `tree_sitter_rust`. The lane wrapper records the exact command, starting HEAD, dirty status, times, exit code and counts. The generated JSON records each complete source expression and its source coordinates, including assertions. Read the inline proofs in the corresponding checkout. It is evidence for that exact checkout, not a second maintained proof source.

Retained sites are confined to concrete construction or local control-flow proofs: immutable closed native encodings already admitted by artifact construction; fixed-size native hash tuples; private sorted map/column/tree algorithms; immediate Some/variant checks under the same exclusive borrow or mutex; and non-cloneable obligations, reservations and performed witnesses whose constructors install the state that consuming cleanup removes. Runtime registry and allocator proofs name their checked issuers and exclusive guards. Media readmission does not directly populate those private bookkeeping states.

A Full demand is not a proof for a later infallible cold lookup. Neither eager commit indexes nor a plausible id/owner invariant prove a payload is resident or a successor exists. Those cases now return typed refusals or carry the actual admitted value. The public cold-read matrix covers all three StorageAccess adjacency doors for every existing region/history/symbol failure class, and a cold/warm court checks their exact relation answers.

Generation publication now has an eager recovery construction proof: cold installation advances the checked issuer and catalog frontier from validated manifest metadata before any reservation. Deferred images admit only generation identities below their captured frontier. Every catalog insertion advances its durable high-water mark, including admitted batches and recovery; reclamation retains it. Abandoned reservations affect only the live cursor, preserving exact cancellation checkpoint identity. The cold frontier court crosses scalar/bulk issuance with direct/admitted publication before any image demand.

The final merged census remains 363 sites (295 panic/extraction forms plus 68 assertions), recorded in target/lane-evidence/landing-panic-census-catalog.log and landing-panic-sites-catalog.json.
