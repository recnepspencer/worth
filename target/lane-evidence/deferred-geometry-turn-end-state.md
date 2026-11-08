# Deferred generation publication: completed scoped handoff

Source commit: df643bdf1debcdea743473fcd952d3416efc8567 (fix(relational): price deferred index publication from live catalog).
Base: 307539feba2f331a8b7406f3a69868bc3ee3b45d. No owner merges or pushes this turn.

## Result and source

The unchanged checkpoint_absence_rebuilds_every_owner_call_like_fresh parent now passes under its original 8M serving allowance. Its former ProviderCommit denial came from quoting future generation publication against usize::MAX, rather than this catalog. Catalog quote/funding now uses checked retained-entry plus batch geometry, protected through actual cutover. Admission pays from the original publication request before the borrowed handoff. Metadata installs once at Native cutover; coordination releases before delayed settlement; owning cleanup stays in the runtime record until settlement and data retires before both funding tickets. Full validation, custody meaning and all original courts remain unchanged.

Sixteen Native files only. Main owners: indexing/generation_catalog/admitted_publication.rs, indexing/prepared_publication.rs, indexing/generation_publication_gate.rs, indexes/authority/maintenance/candidate.rs, mvcc/publication/port_preflight.rs and port_cutover.rs, publication_settlement_registry{,/record}.rs, and authority/commit/pipeline/publication_execution.rs. Denied port.rs and port_handoff.rs were not edited. All edited paths have both exact peer grants; full commands/results are retained. No H5 Query/Store cutover file was edited. Tool-generated UI lock drift was restored to the exact HEAD blob, with no UI commit diff.

## Qualification

Exact per-lane commands, original HEAD/dirty state, start/end/exit, counts and full logs are indexed in deferred-geometry-lane-index.json. All runtime/check logs record base307539 plus frozen WIP. deferred-geometry-source-inputs.json freezes4039 crate Rust files (4032 under src); reviewed-source.json bridges those inputs to committed source blobs. No source changed after verification.

| Selection | Pass | Fail | Selected ignored | Run log |
| --- | ---: | ---: | ---: | --- |
| Native kernels + existing inline-size guard | 7 | 0 | 0 | deferred-geometry-focused/native/run.log |
| Five unchanged Topology parents | 5 | 0 | 0 | deferred-geometry-focused/topology/run.log |
| D18three + graceful seeds0/1 + index-transitionfive | 10 | 0 | 0 | deferred-geometry-focused/store/run.log |
| Bank assessment + transfer parents, default stack | 2 | 0 | 0 | deferred-geometry-focused/bank/run.log |

Native and Bank inventories respectively list31 and1 unselected ignored tests; no ignored body ran. The registered graceful range0..2 is exclusive. No exhaustive, scenario or query-scale suite ran. Each large build checked C: above25GiB; lowest pre-build measurement899728564224bytes. Builds were serial.

Final Native --tests check, fmt, dirty caps(16Rustfiles) and generated context pass. Boundary exits1 with exactly the same three held P-A BC9002 findings as topology-boundary.log. No baseline or exception was added. The32 distinct Native dead-code warnings are unchanged (warning-comparison.json). No broader normal suite, strict lint or M3 certification is claimed.

## Failures and ownership

- Product: usize::MAX deferred quote caused the accepted410016+9932 child quote to exceed414608. Fixed at Native quote/admission/cutover owners; unchanged Topology parent passes.
- Author source: first check had5 diagnostics from Native-cause vs WorkCeilingDenial confusion. Fixed all as one family, preserving exact original request outcomes; next check passes. No test/build ran before the corrected check.
- Author evidence wrapper: first phase-wrapper launch decoded a copied BOM with the shell default encoding and exited1 before any gate. Fixed script prefix/UTF8 source; gates then ran once. No product or runtime-test rerun. The failed-launch stderr used Windows CP1252; its exact original bytes are preserved as base64/SHA256 in raw-log-encoding.json, and only those lines were losslessly transcoded to UTF8 in the readable log. The evidence whitespace audit caught doubled CR from Windows text-mode transcoding; the readable log now has normal CRLF, while the original bytes remain preserved.
- Held P-A: BC9002 cold_source.rs, cold_source/protected_reads.rs and support/durable_medium/runtime_attachment.rs; plus32 Native dead-code diagnostics. They remain untouched and explicitly red/warning-only. Their exact raw evidence is retained.

## Review and budget

Fresh independent reviewer /root/deferred_geometry_final_review: APPROVE; no material findings; no reviewer edits/builds/Cargo/tests. Full verdict and earlier advisory resolutions: deferred-geometry-independent-review.md.

This slice used2 compiling cargo checks(1failed/1passed),0Clippy,4focused builds,4runtime test invocations, and2cached phase-tool cargo runs. Owned runner also issued2 noncompiling check unit-graph inventories and4 test-build unit-graph inventories; these did not invoke rustc. No runtime test reruns. No further Cargo after evidence-only edits. The evidence commit follows source without changing product inputs.

Next: coordinator may consume this source safe point; H5/P-C merges remain separate. This closes only the deferred-geometry slice, not all M3.
