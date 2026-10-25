//! Real binding declaration supplies the approval floor for guarded authoring fixtures.
use super::*;
use crate::application_schema::*;
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

#[derive(serde::Serialize)]
pub(super) struct ProposalInput;

// Authoring validates marker identities; this fixture never installs schema
// references. Implement only the identity contracts it actually consumes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Principal;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExternalMapping;
impl ApplicationEntityMarkerIdentity<TestSchema> for Principal {
    const IDENTIFIER: &'static str = "Principal";
}
impl ApplicationEntityMarkerIdentity<TestSchema> for ExternalMapping {
    const IDENTIFIER: &'static str = "ExternalMapping";
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PrincipalFacts;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MappingFacts;
impl ApplicationAspectMarkerIdentity<TestSchema, Principal> for PrincipalFacts {
    const IDENTIFIER: &'static str = "PrincipalFacts";
    const ASPECT_IDENTITY: AspectIdentity = AspectIdentity(0x9500_0101);
    const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(1);
}
impl ApplicationAspectMarkerIdentity<TestSchema, ExternalMapping> for MappingFacts {
    const IDENTIFIER: &'static str = "MappingFacts";
    const ASPECT_IDENTITY: AspectIdentity = AspectIdentity(0x9500_0102);
    const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(1);
}
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
