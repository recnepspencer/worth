# Basis Capability Lifecycle

Pointers to Admission's basis lifecycle, policy and tenant admission, and
relationship-proof admission entry points, as Host re-exports them.

## Basis lifecycle

- `worth_query_host::facade::admission::basis::basis_lifecycle` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/dx.rs#L80))
- `worth_query_host::facade::admission::basis::BasisLifecycleIntentBuilder` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/dx.rs#L24))
- `worth_query_host::facade::admission::basis::BasisLifecycleIntentDraft` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/dx.rs#L85))
- `worth_query_host::facade::admission::basis::BasisLifecycleDeclarationError` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/declarative.rs#L21))
- `worth_query_host::facade::admission::basis::activate_subscription_basis` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/scoping.rs#L174))
- `worth_query_host::facade::admission::basis::discover_basis_lifecycle_support` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/support.rs#L179))
- `worth_query_host::facade::admission::basis::LowerRuntimeBasisEvidence` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/lower_runtime/mod.rs#L41))
- `worth_query_host::facade::admission::basis::readmit_lower_runtime_evidence` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/lower_runtime/mod.rs#L250))
- `worth_query_host::facade::admission::basis::emit_observation_basis_receipt` ([source](../../crates/worth-query-admission/src/domain_computation/basis_lifecycle/receipts.rs#L259))

## Policy and tenant admission

- `worth_query_host::facade::admission::policy::admit_policy_tenant_context` ([source](../../crates/worth-query-admission/src/domain_computation/policy_basis/admission.rs#L17))
- `worth_query_host::facade::admission::policy::AdmittedPolicyTenantContext` ([source](../../crates/worth-query-admission/src/domain_computation/policy_basis/artifacts.rs#L272))

## Relationship-proof admission

- `worth_query_host::facade::admission::relationship::admit_relationship_proofs` ([source](../../crates/worth-query-admission/src/domain_computation/relationship_proof/admission.rs#L12))
- `worth_query_host::facade::admission::relationship::RelationshipProofDescriptor` ([source](../../crates/worth-query-admission/src/domain_computation/relationship_proof/descriptors.rs#L55))
- `worth_query_host::facade::admission::relationship::RelationshipProofDescriptorSet` ([source](../../crates/worth-query-admission/src/domain_computation/relationship_proof/descriptors.rs#L229))
- `worth_query_host::facade::admission::relationship::RelationshipProofFailureClass` ([source](../../crates/worth-query-admission/src/domain_computation/relationship_proof/errors.rs#L4))
- `worth_query_host::facade::admission::relationship::runtime_backed_relationship_proof_support_profile` ([source](../../crates/worth-query-admission/src/domain_computation/relationship_proof/support.rs#L74))
- Open issue: [#265](https://github.com/recnepspencer/worth/issues/265)
