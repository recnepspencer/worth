# Remaining principal admission boundary

Current evidence: `topology-funding/run.log` selects five exact nonignored parents, 4 pass / 1 fail / 0 ignored. Every capture and restore setup completes. The remaining HAPPY failure is `checkpoint_recovery::computation_partition::oracle::absence::serial::checkpoint_absence_rebuilds_every_owner_call_like_fresh`, at `oracle/demand.rs:92` on the first restored EntryEdit; ConsumerPrincipalBinding answers WorkExhausted under the unchanged 8,000,000-work serving policy. It is not reclassified CLEANUP.

Actual first stop: `principal-charge-debug3.log` resumes the same built binary from a breakpoint at the PDB-verified RVA284821286 for Execution checkpoint_inner closure line150. It records FIRST_WORK_CEILING, then the same genuine failed test, DIAGNOSTIC_EXIT101 / CHARGE_CAPTUREDTrue. No product code or grant is altered. Earlier filename-based probes failed to capture this closure; they are not owner-charge evidence. The offline PDB symbol/line census is `principal-charge-pdb.json`.

Captured call path, in caller order:
- Query principal `resolve_authenticated_principal_with_request` -> snapshot bounded field lookup (limit2);
- Native `indexes/access/pinning/snapshot_field.rs:35` -> pinned read capability;
- `visibility/materialization/read_records/reader/pinned_read/pinning.rs:170` -> `ResidentVisibilitySnapshotBasis::admit`;
- `visibility/snapshot_states/resident_basis.rs:20` -> `branch/root_region_access.rs:115,181` -> demand all Full regions;
- `storage/residency/region_load.rs:240` -> `required_symbols::collect_for_request` -> `collect_head_using`, field-revision loop at `required_symbols.rs:42`;
- `storage/residency/region_load/materialization_funding.rs:201` -> original request checkpoint -> actual WorkCeiling branch.

The charge at required_symbols.rs:42 was added by bfd6d1b240e. The existing 8M fixture budget dates to20a4797833f. The resident Full proof is retained by accepted Native owner code (current resident_basis.rs last changed7b6ac1ad6e); it cannot be forged from Head reads. This establishes charge attribution, not an incorrect charge amount or a measured minimum replacement grant. The collection callback prices each visit as1+22*ilog2(symbol_count+1); possible excess pricing for empty revision slots is a candidate for the Native owner to measure, not an established fix here.

Independent source review traced both candidate doors to the same Full basis. The scoped Head entity read is not a substitute for that Full proof. Query bootstrap already reads latest principal/equality generations, warming the index image, but Native intentionally leaves root contents Cold. Hence blaming initial index-image decoding or claiming repeated full reconstruction after warm retention would be unsupported.

Scope/decision: the current grant assigns Query/Native checkpoint files for the worker fix. This remaining path is Native storage/root/index admission, including P-C's root seams, rather than checkpoint transport. The working agreement forbids editing outside scope and requires a boundary ruling instead of an approximation. An asynchronous scope question asks whether the Native owner takes this slice or integration is granted it. No serving cap increase, eager full-root warmup policy, candidate-only replacement proof, field/receipt assertion weakening or baseline exception is made while that is pending. Options need owner assessment: correct actual symbol-closure work if wrong; lawful candidate-scoped Native admission preserving Full-envelope authentication; or explicitly funded bootstrap preparation / a genuinely derived serving grant. None is silently selected.
