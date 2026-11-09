//! One declared application owns model and versioned proposal meaning.
#[path = "schema/records.rs"]
pub(super) mod storage_model;
pub use storage_model::{
    ExternalIdentityField, ExternalMapping, MappingStatus, MappingTarget, Model, ModelName,
    ModelProposal, ModelTarget, Principal, PrincipalIdentity, Proposal, ProposalData, ProposalName,
    ProposalRecords, Registry, RegistryName, Retention, Target, TargetName,
};
use worth_query_decl::facade::{application_schema::*, authentication::*};

pub mod data;
use data::{StoredProposal, StoredProposalBinding};

use worth_query_decl::facade::application_schema::fragment::{self, *};
#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.consumer.proposals";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}

impl ApplicationSchemaComposition for Schema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("proposal-application.contribution.v1");
    type Members = (ProposalRecords, Contracts);
}
impl Contribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<Schema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<Schema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(super::target_replacement::declare(
            super::revision::declare(super::attempts::declare(super::mutation_access::declare(
                super::retention::declare(super::access::declare(super::storage_query::declare(
                    super::model_query::declare(super::recording::declare(
                        super::creation::declare(
                            schema.principal_binding(PrincipalBinding::reference()),
                        ),
                    )),
                ))),
            ))),
        ))
    }
}

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentity as RecordSelection>::Value,
        <PrincipalIdentity as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
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
