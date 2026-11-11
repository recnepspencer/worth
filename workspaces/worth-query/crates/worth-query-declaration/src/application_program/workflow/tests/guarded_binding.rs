//! Real binding declaration supplies the approval floor for guarded authoring fixtures.
use super::*;
use crate::application_schema::*;
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

#[derive(serde::Serialize)]
pub(super) struct ProposalInput;

use crate::application_schema::fragment::{self, *};
pub(super) struct WorkflowActors;
impl Fragment for WorkflowActors {
    const NAME: &'static str = "workflow_actors";
    type Bindings = ActorBindings;
    type Members = (Principal, ExternalMapping);
}
pub(super) struct ActorBindings;
impl BindingSet for ActorBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for ActorBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for ActorBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ActorBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}
pub(super) struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalData;
}
pub(super) struct PrincipalData(u64);
impl fragment::Record<PrincipalData> for ActorBindings {
    fn record() -> impl Shape<PrincipalData, Self> {
        field("PrincipalId", |row: &PrincipalData| row.0)
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalData(id))
            })
    }
}
pub(super) struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingData;
}
pub(super) struct MappingData {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingData> for ActorBindings {
    fn record() -> impl Shape<MappingData, Self> {
        field("ExternalId", |row: &MappingData| row.identity.clone())
            .equality_queryable(true)
            .and("MappingStatus", |row: &MappingData| row.status)
            .equality_queryable(true)
            .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingData { identity, status })
            })
    }
}
pub(super) type PrincipalId = FieldSelection<WorkflowActors, Principal, PrincipalFacts, u64>;
pub(super) type ExternalId = FieldSelection<
    WorkflowActors,
    ExternalMapping,
    MappingFacts,
    WorthQueryExternalPrincipalIdentity,
>;
pub(super) type MappingStatus =
    FieldSelection<WorkflowActors, ExternalMapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub(super) struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    fn reference() -> ApplicationRelationRef<TestSchema, Self, ExternalMapping, Principal> {
        relation(WorkflowActors, ExternalMapping, Self)
    }
}
pub(super) struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        TestSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalId as RecordSelection>::Value,
        <PrincipalId as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalId::at("ExternalId"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalId::at("PrincipalId"),
                ),
            },
        )
    }
}

crate::worth_query_mutation_binding!(@binding
    pub(super) ApplyBinding for ProposalInput, schema TestSchema,
    identity "worth.query.tests.workflow.apply-binding.v1",
    input ProposalInputBinding, operation ApplyChange, result BoolResultBinding,
    idempotency u64, identity "worth.query.tests.workflow.apply-key.v1",
    decision bool, denial BoolResultBinding,
    handler identity "worth.query.tests.workflow.apply-handler.v1",
    program , workflow_authority required,
    workflow_requirements crate::application_operation::ApplicationWorkflowExecutionRequirements::requiring_approval::<ChangeApprover>(),
    inspection_disclosure ,
    outputs crate::application_operation::NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope_binding crate::application_operation::ApplicationMutationFieldScope<PrincipalId>,
    scope_field_type PrincipalId,
    scope_field PrincipalId::at("PrincipalId"),
);

#[test]
fn guarded_binding_members_keep_their_declared_identities() {
    let principal = PrincipalId::at("PrincipalId");
    let mapping = ExternalId::at("ExternalId");
    assert_eq!(principal.entity(), "Principal");
    assert_eq!(mapping.entity(), "ExternalMapping");
    assert_eq!(
        principal.aspect(),
        aspect(WorkflowActors, Principal, PrincipalFacts).name()
    );
    assert!(principal.checked().unwrap().locus().field() == "PrincipalId");
    assert_eq!(
        mapping.aspect(),
        aspect(WorkflowActors, ExternalMapping, MappingFacts).name()
    );
}
