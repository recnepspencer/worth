//! Real binding declaration supplies the approval floor for guarded authoring fixtures.
use super::*;
use crate::application_schema::*;
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

#[derive(serde::Serialize)]
pub(super) struct ProposalInput;

crate::worth_query_entity!(pub(super) Principal for TestSchema);
crate::worth_query_entity!(pub(super) ExternalMapping for TestSchema);
crate::worth_query_aspect!(pub(super) PrincipalFacts for TestSchema, Principal;
    identity = AspectIdentity(0x9500_0101), revision = AspectContractRevision(1),);
crate::worth_query_aspect!(pub(super) MappingFacts for TestSchema, ExternalMapping;
    identity = AspectIdentity(0x9500_0102), revision = AspectContractRevision(1),);
crate::worth_query_field!(pub(super) PrincipalId for TestSchema, Principal, PrincipalFacts:
    u64 => U64ApplicationValueBinding, read_only, equality);
crate::worth_query_field!(pub(super) ExternalId for TestSchema, ExternalMapping, MappingFacts:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding,
    read_only, equality);
crate::worth_query_field!(pub(super) MappingStatus for TestSchema, ExternalMapping, MappingFacts:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding,
    read_write, equality);
crate::worth_query_relation!(pub(super) MappingTarget in TestSchema, ExternalMapping => Principal;
    integrity = same_context_unbounded_retain_dangling);
crate::worth_query_principal_binding!(pub(super) PrincipalBinding in TestSchema,
    mapping ExternalMapping {
        identity: ExternalId, status: MappingStatus,
        target: MappingTarget => Principal, principal_identity: PrincipalId
    }
);
crate::worth_query_mutation_binding!(@binding
    pub(super) ApplyBinding for ProposalInput, schema TestSchema,
    identity "worth.query.tests.workflow.apply-binding.v1",
    input ProposalInputBinding, operation ApplyChange, result BoolResultBinding,
    idempotency u64, identity "worth.query.tests.workflow.apply-key.v1",
    decision bool, denial BoolResultBinding,
    handler identity "worth.query.tests.workflow.apply-handler.v1",
    program , workflow_authority required,
    workflow_requirements crate::application_operation::ApplicationWorkflowExecutionRequirements::requiring_approval::<ChangeApprover>(),
    outputs crate::application_operation::NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope_binding crate::application_operation::ApplicationMutationFieldScope<
        TestSchema, Principal, PrincipalFacts, PrincipalId, u64, ReadOnly, NoApplicationUnit
    >,
    scope_field_type Principal, PrincipalFacts, PrincipalId, u64, ReadOnly, NoApplicationUnit,
    scope_field PrincipalId::reference(),
);

#[test]
fn guarded_binding_members_keep_their_declared_identities() {
    assert_eq!(Principal::reference().name(), "Principal");
    assert_eq!(ExternalMapping::reference().name(), "ExternalMapping");
    let principal_facts = PrincipalFacts::reference();
    assert_eq!(principal_facts.name(), "PrincipalFacts");
    assert_eq!(principal_facts.identity(), AspectIdentity(0x9500_0101));
    let mapping_facts = MappingFacts::reference();
    assert_eq!(mapping_facts.name(), "MappingFacts");
    assert_eq!(mapping_facts.identity(), AspectIdentity(0x9500_0102));
}
