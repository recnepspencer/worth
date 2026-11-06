//! The decision-read schema: one principal binding and one contribution that
//! owns the Source, the managed-index Parent and the decision-read operation.
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
#[path = "records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexDefinition;

use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

/// The contribution identity that owns the managed index and its provider.
pub const MEMBERS: &str = "worth.query.tests.decision-read-entry.members.v1";

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct DecisionReadSchema;
impl Fragment for DecisionReadSchema {
    const NAME: &'static str = "DecisionReadSchema";
    type Bindings = NoBindings;
    type Members = (DecisionReadMembers,);
}
impl ApplicationSchema for DecisionReadSchema {
    const OWNER: &'static str = "worth.query.tests.decision-read-entry";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_declaration::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl DecisionReadSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for DecisionReadSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct DecisionReadMembers;
impl ApplicationSchemaContribution<DecisionReadSchema> for DecisionReadMembers {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.tests.decision-read-entry.members.v1",
        );
    type Members = (DecisionRecords, Contracts);
}

pub struct Contracts;
impl Registration<DecisionReadSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<DecisionReadSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DecisionReadSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
                .principal_binding(DecisionPrincipalBinding::reference())
                .application_query(crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::source())
                .application_query_binding::<crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::SourceBinding>()
                .application_query(crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::parent())
                .application_query_binding::<crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::ParentBinding>()
                .managed_index(ApplicationManagedIndexDefinition::<
                    DecisionReadSchema,
                    crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::DecisionIndex,
                >::new());
            crate::domain_computation::primary_graph::tests::fixture::decision_read_world::operation::declare(schema)
        })
    }
}

pub struct DecisionPrincipalBinding;
impl DecisionPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DecisionReadSchema,
        Self,
        Mapping,
        Principal,
        <PrincipalIdentity as RecordSelection>::Value,
        <PrincipalIdentity as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DecisionPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}
