# Relational clean-cutover audit

The starting revision is `6a135ca5a2`. The production module graph is followed from `src/lib.rs`, honoring `#[path]` and excluding `#[cfg(test)]` modules/items and test functions. The census counts method calls to `expect` and `unwrap`, and `panic!` / `unreachable!`, including expressions inside macro argument token trees. It does not count `unwrap_or`, `unwrap_or_else`, `Arc::try_unwrap`, assertions, or test/example code. Nested calls count separately. Platform-specific production branches are counted conservatively.

Starting census: **458** sites. Final census: **358** sites. **100** sites removed. The earlier intermediate census omitted two predecessor checks inside `format!`; both now use `if let` bindings.

Construction now returns `RelationalRuntimeConstructionDenial` before dispatch. Symbol and derived-index identity frontiers refuse without advancing or wrapping. Projection contract and historical visibility failures retain typed causes through Query. Storage reconstruction failures use their existing storage/residency classifications; guard admission, lease ownership, discovery rounds and damage stickiness retain their existing rules.

## Removed variants

Some declarations never had a production producer. For these there is no producer-retirement commit to name: the table records that fact instead of inventing one. `0bc13edff1` imports the implementation into the reviewed history with those declarations already orphaned. The historical source introductions are supplied where established.

| Removed variant | Producer retirement / historical evidence |
| --- | --- |
| `PreparationFailureClass::PacketOverlapDetected` | `b6deb60e22`: removed synthetic diff/worker preparation-fault producers and their old test controls. |
| `PreparationFailureClass::WorkerEvaluationFailure` | `b6deb60e22`: removed synthetic diff/worker preparation-fault producers and their old test controls. |
| `PreparationFailureClass::FragmentCanonicalizationFailure` | `b6deb60e22`: removed synthetic diff/worker preparation-fault producers and their old test controls. |
| `PreparationFailureClass::ConsumerFailureNonAuthoritative` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `RelationalAuthorizationPlanDenial::PathStartsAtWrongKind` | `54a77447e6`: removed the first-traversal special case; traversal continuity supplies `DiscontinuousTraversal`. |
| `RelationalBranchCellDenial::BranchIdentityMismatch` | Declaration-only orphan in the cell introduced by `3d29c4fa93`; no producer-retirement commit exists. Runtime affinity and checkpoint observation have their live typed denials. |
| `RelationalBranchCellDenial::CheckpointRuntimeMismatch` | Declaration-only orphan in the cell introduced by `3d29c4fa93`; no producer-retirement commit exists. Runtime affinity and checkpoint observation have their live typed denials. |
| `RelationalBranchCellDenial::CheckpointRootReadmissionRequired` | Declaration-only orphan added by `78291e2f63`; no producer-retirement commit exists. Actual checkpoint readmission returns its native refusal. |
| `MergeExecutionPreparationError::OwnerBinding` | No ordinary producer after the owner-bound request API in `3d29c4fa93`; its residual unit adapter now reports the binding cause through its existing planning/normalization outcome. |
| `MergeExecutionMutationPlanError::MissingSourceHeadEnvelope` | `b6deb60e22`: prepared mutation plans retain their source basis rather than re-reading a source envelope during execution. |
| `MergeExecutionMutationPlanError::MissingTargetRelationSnapshot` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `MergePlanningError::MissingPolicyTargetRecord` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `CompanionPreflightStop::WorkExhaustedWithoutRequirement` | `ee07da2b0e`: replaced preflight forecast quotas with carried actual request accounting; real execution stops use `Execution(cause)`. |
| `CompanionPreflightStop::PreparationMemoryExhausted` | `ee07da2b0e`: replaced preflight forecast quotas with carried actual request accounting; real execution stops use `Execution(cause)`. |
| `CompanionPreflightStop::RetainedCompanionCapacityExhausted` | `ee07da2b0e`: replaced preflight forecast quotas with carried actual request accounting; real execution stops use `Execution(cause)`. |
| `EntityAuthoritativeAspectStateDenial::MissingAspectPlan` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `EntityAuthoritativeAspectStateDenial::ContractValidationDenied` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `EntityAuthoritativeAspectStateDenial::UnsupportedAspectFieldTarget` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `EntityAuthoritativeAspectStateDenial::StructValueConstructionDenied` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `EntityAuthoritativeAspectStateDenial::StructContractValidationDenied` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `EntityAuthoritativeAspectStateDenial::StateAdmissionDenied` | `a707a48b1b`: removed field-creation aspect planners and lowered authoring through admitted record patches. The obsolete field-target presentation test was retired with that producer. |
| `ReplayFailureClass::UnsupportedReplaySchema` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `SubscriberStreamFailureClass::UnknownCheckpoint` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `SubscriberStreamFailureClass::RetainedHistoryUnavailable` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `SchemaRegistryErrorClass::SchemaVersionMismatch` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `RelationalPublicationDenial::BranchUnavailable` | No production constructor in the imported implementation (`0bc13edff1`); declaration/match-only residue, also present in the earlier source lineage (`6af9e15997`). No producer-retirement commit exists. |
| `CompanionPreflightStop::WorkExhausted` | `ee07da2b0e`: replaced preflight forecast quotas with carried actual request accounting; real execution stops use `Execution(cause)`. |

Consumer-only residue retired with the same preflight forecast producer (`ee07da2b0e`): `PreflightReadRefusal::RetainedCompanionCapacityExhausted` and `RetainedFactStop::Memory`.

Forecast-only Query presentation rows were removed or replaced with rows for surviving actual counter refusals. These rows tested translation of the deliberately retired variants, not live quota enforcement. The required-hints journey now requires both hints to succeed instead of accepting a retired forecast-capacity deferral. No test was ignored or removed to bypass a live product claim.

## Retained sites and proofs

The complete [retained panic-site proof appendix](CUTOVER_PANIC_PROOFS.md) lists all 358 sites and the construction/type proof adjacent to each retained assertion.
