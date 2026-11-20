# Basis Capability Lifecycle

Reference for Admission's basis lifecycle, policy and tenant admission, and
relationship-proof admission. Every statement below is a signature, an enum
variant or a check made in the linked source.

## Basis lifecycle

Path: `worth_query_host::facade::admission::basis`, which re-exports
Admission's `basis_lifecycle` module ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/mod.rs)).

- `basis_lifecycle() -> BasisLifecycleIntentBuilder`
  ([dx.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/dx.rs)).
- Builder methods returning `BasisLifecycleIntentDraft`, with their parameter
  names ([dx.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/dx.rs),
  [declarative.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/declarative.rs)):
  `current_head()`, `branch_head(branch_identity, accessible: bool)`,
  `branch_snapshot(branch_identity, snapshot_identity)`,
  `preview(preview_identity)`,
  `runtime_snapshot(snapshot_identity, lower_runtime_binding_digest)`,
  `historical_snapshot(snapshot_identity, replay_supported: bool)`,
  `historical_commit(commit_identity, replay_supported: bool)`,
  `tenant_scoped(tenant_identity, branch_identity, schema_identity, tenant_schema_matches: bool)`.
  `policy_scoped(policy_digest, tenant_identity, branch_identity, schema_identity)`
  returns a `BasisLifecyclePolicyIntentDraft`. The `bool` parameters are
  caller-supplied facts that the intent records.
- Draft methods, each returning `Result<_, BasisLifecycleDeclarationError>`:
  `observe()` → `ScopedObservationBasis`, `prepare_mutation()` →
  `ScopedMutationPreparationBasis`, `replay()` → `ScopedReplayBasis`,
  `inspect()` → `ScopedInspectionBasis`, `materialize()` →
  `ScopedMaterializationBasis`, `declare_subscription()` →
  `ScopedSubscriptionDeclarationBasis`, `activate_subscription()` →
  `ScopedSubscriptionActivationBasis`, `close_preview()` →
  `ScopedPreviewCloseoutBasis`.
- `activate_subscription_basis(&ScopedSubscriptionDeclarationBasis) -> ScopedSubscriptionActivationBasis`
  ([scoping.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/scoping.rs)).
- `BasisLifecycleDeclarationError::intent_denial() -> Option<&BasisIntentDenial>`
  and `eligibility_denial() -> Option<&DeniedBasisCapability>`.
- `discover_basis_lifecycle_support(family: BasisFamily, operation_lane: &'static str) -> BasisLifecycleSupportDiscovery`
  ([support.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/support.rs)).

Selecting a world, for example with `runtime_snapshot`, records the identities
it is given; it does not observe a snapshot. No method of Host's application
request API (`worth_query_host::facade::application_entry`) takes a scoped
basis, and Host's `causal_inspection` namespace does not export
`ScopedInspectionBasis` (a `compile_fail` doctest in
[causal_inspection_contract.rs](../../crates/worth-query-host/src/causal_inspection_contract.rs)).
Requests select their product as described in the
[Ordinary Application Front Door](../foundations/ordinary-application-front-door.md).

### Lower-runtime evidence

- `LowerRuntimeBasisEvidence::from_runtime_bridge_facade(basis_digest, evidence_digest, retained_evidence_lookup_width: usize)`
  ([lower_runtime/mod.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/lower_runtime/mod.rs)) stores the
  values it is given under the Runtime Bridge authority label.
- `readmit_lower_runtime_evidence(scoped_basis, evidence) -> Result<LowerRuntimeBoundBasis<_>, DeniedBasisCapability>`
  and `emit_observation_basis_receipt(LowerRuntimeBoundBasis<ScopedObservationBasis>) -> BasisUseReceipt`
  ([receipts.rs](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/receipts.rs)).

No production code path constructs this evidence from a Runtime Bridge
observation today, so a receipt built from caller-supplied values is not
evidence of an observed snapshot.

## Policy and tenant admission

Paths: `worth_query_host::facade::admission::{policy, tenant}`
([policy_basis/admission.rs](../../crates/worth-query-admission/src/domain_computation/policy_basis/admission.rs)).

- `policy::admit_policy_tenant_context(query: &CanonicalQueryArtifact, policy: PolicyRuleSnapshot, tenant: TenantBindingSnapshot, branch: BranchAccessGrant, schema: SchemaVariantSnapshot, mode: PolicyExecutionModeRequest) -> Result<AdmittedPolicyTenantContext, PolicyTenantAdmissionError>`.

## Relationship-proof admission

Path: `worth_query_host::facade::admission::relationship`
([relationship_proof/admission.rs](../../crates/worth-query-admission/src/domain_computation/relationship_proof/admission.rs)).

- `admit_relationship_proofs(query: &CanonicalQueryArtifact, admitted: &AdmittedPolicyTenantContext, descriptor_set: &RelationshipProofDescriptorSet) -> Result<(RelationshipProofAdmission, RelationshipProofCounters), RelationshipProofError>`.
- Descriptor constructors on `RelationshipProofDescriptor`: `direct_edge`,
  `bounded_ancestor`, `bounded_descendant`, `tenant_membership`,
  `host_callback_forbidden` ([descriptors.rs](../../crates/worth-query-admission/src/domain_computation/relationship_proof/descriptors.rs)).

Checks the function makes, with the `RelationshipProofFailureClass` it
returns:

- the topology descriptors must cover the query's traversal exactly, each step
  matched by a descriptor for the same relation with enough depth and every
  topology descriptor used: `QueryShapeMismatch`;
- descriptor count above the budget's `max_descriptors`, or total topology
  width above its `max_topology_width`: `RelationshipProofBudgetExceeded`;
- a bounded ancestor or descendant descriptor with `max_depth` 0:
  `UnboundedProofTopology`;
- a descriptor policy digest different from the admitted context's:
  `PolicyMismatch`; a tenant-membership digest different from the admitted
  tenant schema: `TenantSchemaMismatch`;
- recursive-walk and host-callback descriptors: `UnboundedRecursiveWalk`,
  `HostCallbackForbidden`.

It does not check that `admitted` was admitted for the same `query`: the
result combines the passed query's digest with the context's policy and tenant
digests. Callers must pass the context admitted for that query.

`runtime_backed_relationship_proof_support_profile()`
([support.rs](../../crates/worth-query-admission/src/domain_computation/relationship_proof/support.rs)) reports descriptor admission
and the four topology surfaces as `Verified`, runtime proof evaluation as
`Deferred`, and host callback proofs as `Forbidden`.

## Related docs

- [Cross-Runtime Causal Inspection](./cross-runtime-causal-inspection.md)
- [Application Authorization And Emergency Elevation](./application-authorization-and-emergency-elevation.md)
