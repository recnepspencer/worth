# Align 9.19 with completed 9.18.1 master

The user explicitly requested another latest-master merge after 9.18.1 completed,
before continuing 9.19 implementation. Fetch selected origin/master
`38e679ec4ab4226db45f0ee33e3a93d42bddf4a3`; branch HEAD before merge was
`1789f2fd0e`. All uncommitted 9.19 checked-scale work, including untracked files,
was preserved in stash `cb0b7ff5594bbc78ed4578886e0f916f13a8687e` before merging.

## Boundary findings and resolutions

- Keep 9.19's one shared Source permission owner and same-scope admissions.
  Carry master's `with_original_settled_producer` handoff at the actual shared
  `with_resolved_plan` owner, before parameter/permission/session admission.
  Do not restore a duplicate Source read pipeline from a textual conflict.
- Preserve distinct archive tags: invariant managed Source 49, scalar-storage
  qualifier 50. Both original encoders/decoders and transported meanings remain.
- Combine ManagedSourceRead/Key and OutputFieldCaptureFailure exports. Retain
  the specialized managed-index facade and one copy of master's marking controls.
- Combine exact control-callers and private-owner audit entries. The combined
  audit has 51 distinct source/test entries and no conflicting shared entries.
  Do not replace exact placement rulings with blanket allowances.
- Keep master's actual selected-head retention and physical allocation visitor.
  Allocation-address deduplication is pinned diagnostic/storage evidence; it
  does not replace canonical identity, semantic occurrences or read admission.
- The new result-presence contract distinguishes OptionalAspect from an optional
  child. The managed Native Source receiver still flattens the two absences into
  Option; only `schema_presence() == Optional` may produce Null there. Explicitly
  reject OptionalAspect maintained Source shapes as UnsupportedInvariantSource.
  Ordinary Query/public-cache Source reads retain master's lossless interpreter.
  Full maintained scalar/OptionalAspect integration needs a later qualified
  lossless Native receiver, not a compile-only permissive conversion.

The running checked-scale binary was built before this merge and continued
without interruption. Its result is premerge evidence. It passed membership and
all Source/callback obligations, but still failed the unchanged remote cost guard:
1k `(2919061,2611311)`, 2k `(3149263,2849684)`, 2k+50k
`(5360656,5110305)`, 1083.09s. It cannot certify the merged implementation or
close the cost obligation. Preserve the failed result and original thresholds.

## Verification and next work

Run the resolved Query execution compile, the existing committed managed-index
consumer family and the archive member tests (49/50 included), plus scoped
boundary/context/format/line/function/whitespace checks. The independent reviewer
gates this merge before commit. Restore the preserved WIP only after that merge
commit; keep the stash until restoration has been verified.

Then continue the current 9.19 slice. The user's additional instruction requires
concrete scoped reuse of shared immutable work at its actual identity/version
and authority scope, retaining incoming-edge/occurrence multiplicity and every
admission. Inspect repeated NodePages copies and repeated validation/coverage
work with small real growth/shared-subgraph cases before another large run.
Do not add a speculative cache framework, a global semantic-id cache, lowered
quotes or relaxed acceptance thresholds. Phase 3 and all later phases remain
unfinished.

## Merge verification (2026-10-30)

Verified on the merged sources with the configured Rust 1.99.1, default cache
and existing profiles; no feature/target/profile overrides:

- Query execution `cargo check`: passed, 1m06s.
- Public committed managed-index family, skipping only the separate large court:
  43 passed in 3.63s after a 2m46s build. This exercises Source permission,
  generated materialization, pending publication/adoption and retained affinity.
- `worth-query-package-archive --lib`: all 167 passed in 0.01s, including the
  invariant Source and scalar-storage wire members.
- Already built proposal storage/typed-reload smoke: 2 passed in 1.44s.
- Fresh `cargo run` boundary-check and agent-context check: passed.
- Dirty Rust line caps: 774 files passed. Function scrutiny: 761 files,
  454 advisories and zero scan errors; the accepted incoming master scope remains
  distinct from the reviewed conflict/compatibility transitions.
- Rustfmt on resolved/compatibility Rust files and whitespace checks: passed.

Independent qa-loop, qa-tests and code-quality-qa gate the merge commit. These
checks do not qualify the uncommitted scale repair or erase its failed SDK guard.
