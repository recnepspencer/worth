//! Issuance retains the grant facts; admission-only resource custody ends here.
use super::*;

pub(in super::super::super) fn admit_observed_capability<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Capability,
    Operation,
    Input,
>(
    observed: ObservedCapabilityAdmission<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >,
) -> WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let ObservedCapabilityAdmission {
        prepared,
        resolved,
        authorization,
        ..
    } = observed;
    let product = prepared
        .graph_work
        .authorization_product()
        .expect("observed capability admission owns its authorization Product");
    let product_description =
        crate::basis::WorthQueryProductBranchReadIdentity::from_observation(product.observation())
            .product_branch();
    let security_basis = product.relational_basis().admission_identity().clone();
    let input = prepared.input;
    let canonical_work = prepared
        .capability
        .lookup_evidence()
        .canonical_work()
        .combine(WorthQueryCanonicalWorkEvidence::streamed_identities(
            prepared.governed_input_work,
        ));
    WorthQueryAdmittedApplicationCapabilityAccess {
        authentication: prepared.principal.retain_authentication(),
        runtime_authority: prepared.graph_work.runtime_authority(),
        home_owner: prepared
            .runtime
            .runtime
            .product_runtime
            .owner
            .owner_identity(),
        binding_identity: prepared.graph_work.binding().clone(),
        capability: prepared.capability.contract().name().into(),
        capability_type: prepared.capability.contract().capability_type().into(),
        operation: prepared.capability.contract().operation().into(),
        principal_entity_id: prepared.graph_work.principal(),
        input,
        governed_input_identity: Some(prepared.governed_input_identity),
        projection: prepared.projection,
        resolved,
        authentication_valid_until: prepared.principal.valid_until(),
        request_scope: prepared.request_scope,
        canonical_work,
        authorization,
        operation_admission_identity: prepared.operation_admission_identity,
        graph_work_session: prepared.graph_work.identity(),
        graph_work_managed_run: prepared.graph_work.managed_run_identity(),
        graph_work_branch: prepared.graph_work.branch().relational().clone(),
        graph_work_capability: prepared.graph_work.capability_access_context(),
        graph_work_decision_facts: prepared.graph_work.retained_decision_facts(),
        product: product_description,
        security_basis,
        _marker: PhantomData,
    }
}
