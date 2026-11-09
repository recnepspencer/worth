# Restoration header validation report

Public issue [#74](https://github.com/recnepspencer/worth/issues/74), measured
2026-11-05 against master `0173a4da614eb01ddcfc24e3245d4e2ae032d41e`.

Origin selection now requires an immutable envelope view minted by its header
validator. Installation and historical reconstruction retain that view while
selecting incident entities, relations and routes. Restoration validation builds
keyed membership indexes and shares source-ancestry traversal across publications.

The public Native journey creates ten chained publication groups, grows each
from eight to sixteen output records and from 36 to 72 relations, retires their
authored source, then restores it and propagates all ten writers. Its grown
output contains 160 records and 720 relations. Inter-group incidence exercises
relation creation classification on the recorded restoration envelope.

| Phase | Master seconds | Changed seconds |
| --- | ---: | ---: |
| Create | 0.726517 | 0.655811 |
| Grow | 1.625530 | 1.530845 |
| Retire | 0.917224 | 0.857633 |
| Undo retirement and propagate | 13.436118 | 3.944273 |

These are single wall-clock samples, excluding compilation, from an in-memory
runtime in the unoptimized `test` profile. Other lanes were building concurrently.
The host was an arm64 Apple M5 Ultra with 30 logical CPUs and 96 GiB RAM, macOS
27.2, using Rust 1.99.1. The samples describe this workload; they establish no
timing budget. The baseline ran the identical journey helpers with master
production code, then restored its temporary test registration by SHA-256.

Reproduce the journey with:

```sh
cargo test -p worth-court-relational-certification \
  --test worth_court_relational_certification \
  public_create_grow_retire_restore_relation_scale_report \
  -- --nocapture --test-threads=1
```

The separate count court observes actual restoration-validator entries and its
charged work. Increasing lookup repetitions leaves the count at one. Doubling
each independent input dimension produces these work counts:

| Dimension | Size 2 | Size 4 | Size 8 |
| --- | ---: | ---: | ---: |
| Publications | 391 | 779 | 1555 |
| Members per publication before growth | 391 | 479 | 655 |
| Relations per publication before growth | 415 | 479 | 607 |

The court checks linear growth increments rather than elapsed time. Restoring
per-lookup validation makes its count assertion fail. Removing entity mapping
membership makes the corruption court fail. Both mutations were restored by
SHA-256 before qualification; the corruption court requires refusal before any
origin lookup or publication effect.
