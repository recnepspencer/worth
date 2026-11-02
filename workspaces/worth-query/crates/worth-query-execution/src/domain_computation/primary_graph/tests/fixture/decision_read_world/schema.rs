//! The decision-read schema: one principal binding and one contribution that
//! owns the Source, the managed-index Parent and the decision-read operation.
use super::{operation, queries};
use worth_query_declaration::facade::application_query::ApplicationManagedIndexDefinition;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::{
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_principal_binding,
    worth_query_relation,
};

/// The contribution identity that owns the managed index and its provider.
pub(in crate::domain_computation::primary_graph::tests) const MEMBERS: &str =
    "worth.query.tests.decision-read-entry.members.v1";

worth_query_declaration::worth_query_application! {
    pub DecisionReadSchema {
        owner: "worth.query.tests.decision-read-entry", version: (1, 0),
        contributions: [DecisionReadMembers],
    }
}
worth_query_declaration::worth_query_application_contribution! {
    pub contribution DecisionReadMembers in DecisionReadSchema {
        identity: "worth.query.tests.decision-read-entry.members.v1",
        members: |schema| {
            let schema = schema
                .entity(Mapping::reference())
                .entity(Principal::reference())
                .aspect(Mapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .field(Mapping::reference(), ExternalIdentityField::reference())
                .field(Mapping::reference(), MappingStatus::reference())
                .field(Principal::reference(), PrincipalIdentity::reference())
                .relation(
                    MappingTarget::reference(),
                    Mapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(DecisionPrincipalBinding::reference())
                .application_query(queries::source())
                .application_query_binding::<queries::SourceBinding>()
                .application_query(queries::parent())
                .application_query_binding::<queries::ParentBinding>()
                .managed_index(ApplicationManagedIndexDefinition::<
                    DecisionReadSchema,
                    queries::DecisionIndex,
                >::new());
            operation::declare(schema)
        }
    }
}

worth_query_entity!(pub Mapping for DecisionReadSchema);
worth_query_entity!(pub Principal for DecisionReadSchema);
worth_query_aspect!(pub ExternalIdentity for DecisionReadSchema, Mapping; identity = AspectIdentity(0x91a0_2001), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for DecisionReadSchema, Principal; identity = AspectIdentity(0x91a0_2002), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for DecisionReadSchema, Mapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatus for DecisionReadSchema, Mapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_field!(
    pub PrincipalIdentity for DecisionReadSchema, Principal, PrincipalFacts:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub MappingTarget in DecisionReadSchema,
    Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub DecisionPrincipalBinding in DecisionReadSchema,
    mapping Mapping {
        identity: ExternalIdentityField,
        status: MappingStatus,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentity
    }
);
