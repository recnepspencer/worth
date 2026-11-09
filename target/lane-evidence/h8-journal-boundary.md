# H8 next slice: dispatch journal and retirement boundary

Base 5d41be2990. Independent read-only boundary review h8_boundary_review, 2026-11-08. The completed first outbox-rebinding slice does not enable durable dispatch. No World or Native edits are authorized.

## Lawful route

QueryExecution owns dispatch semantics and its medium contract. Host implements that contract in application_home against a narrow owned query.dispatch operation issued by StoreApplication from the SAME WorldReadmittedApplicationHome. Existing dependencies support this direction; road1.toml forbids a QueryExecution -> Store dependency and confines Store naming to Host application_home. Use the checkpoint operation's actual-home Arc::ptr_eq retirement as the lifetime analogue, not HomeMaintenance. Pending physical publications and their SAME recovery retainers remain in the owning task on refusal. No unrestricted journal-name or physical-runtime door.

The unheld fresh/existing Host preparation -> bootstrap.graph -> Query installation preflight route can carry this owner through D8. It does not discharge the following protected export/retirement seams.

## Protected files that the coherent slice needs

- QueryExecution src/facade/application_home_integration.rs:1-17. This is the existing closed Host composition facade, mounted by facade.rs and used by Host opening.rs. It must export the dispatch medium, attachment and readmission types from their actual owner. P-C diff against e787b51b3e exits 1 AND its status is M. It is not granted. Adding another integration facade or alias would bypass this owner, so no substitute surface is proposed.
- QueryExecution src/domain_computation/primary_graph/application_runtime/home_retirement.rs::preflight_supplied_home_owners (currently 123-145). Nonempty inbound verifier ownership and any external transport installation unconditionally refuse close. H8 requires real owner admission: executing work refuses; durably owned quiescent custody survives; fresh host configuration retires honestly. Deleting these checks is not a fix. P-C diff exits 1; status is clear. A new runtime phase also needs the existing RetiringPrimaryGraphRuntime::advance progression in this file. The granted provider-owned retirement path can retire a provider-owned medium but cannot bypass this earlier external-owner check.

These files need a coordinator grant or P-C's reviewed merge before this dispatch/custody slice can be completed. They have not been edited. Preserve H5's held Query/Store files and both World medium retired variants for the merge of both owners.

## Original meaning across transitions

Original program activation is read at the original retained Native basis via program_occurrence_gate::resolve_at_lease and the authentic rostered rendering/recovery/retain_original_meaning APIs. That program inventory does not contain an old compiled operation catalog. Its roster entries share one installed schema binding. Therefore slot lookup or an old program revision cannot recreate a changed old inbound contract. Keep the original persisted operation-definition comparison. A changed old contract needs authentic retained installed operation meaning or an explicit transition disposition/refusal; never substitute current meaning.

## Independent work inventory

The portable identity owners (external_effect/{identity_derivation,causal_event,dispatch}.rs and observed_source/source_identity{,/runtime_commitment,/interning,/identity_kind,/tests/epoch_construction}.rs) pass the P-C per-file grant probes in h8-portable-identity-grants.json. No source edits were made there while the first slice's validation was frozen. Their process-local digest inputs, durable dispatch/inbound/claims/R13, graceful close and the Bank/oracle/docs cutover remain unfinished H8 requirements. H8 is not landed.
