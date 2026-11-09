//! Authored root/child values and the counter changed only after admitted reads.
#[path = "schema/records.rs"]
mod storage_model;
pub use super::{
    binding::NestedBinding, opaque::SwallowingValue, operation, ordinary_attempt, query,
    repeated_query,
};
pub use storage_model::*;
pub use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    *,
};

use worth_query_decl::facade::application_schema::fragment::{self, *};
#[derive(Clone, Copy)]
pub struct AdmissionSchema;
impl Fragment for AdmissionSchema {
    const NAME: &'static str = "AdmissionSchema";
    type Bindings = NoBindings;
    type Members = (AdmissionContribution,);
}
impl ApplicationSchema for AdmissionSchema {
    const OWNER: &'static str = "canonical_admission";
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
impl AdmissionSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for AdmissionSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct AdmissionContribution;
impl ApplicationSchemaContribution<AdmissionSchema> for AdmissionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("canonical-admission.application.v1");
    type Members = (StorageModel, Contracts);
}
impl AdmissionContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<AdmissionSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<AdmissionSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<AdmissionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<AdmissionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(DocumentPrincipalBinding::reference())
            .operation(
                operation::Advance::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation::Advance::reference(), 32)
            .operation_read_field(
                operation::Advance::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                operation::Advance::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_read_field(
                operation::Advance::reference(),
                CounterField::at("CounterField"),
            )
            .operation_read_relation(operation::Advance::reference(), ChildLink::reference())
            .operation_write(
                operation::Advance::reference(),
                CounterField::at("CounterField"),
            )
            .application_mutation_binding::<operation::AdvanceBinding>()
            .operation(
                ordinary_attempt::ObserveCanonical::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(ordinary_attempt::ObserveCanonical::reference(), 2)
            .operation_read_field(
                ordinary_attempt::ObserveCanonical::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .application_query_binding::<query::ReadBinding>()
            .application_query(query::definition())
            .application_query_binding::<repeated_query::RepeatedBinding>()
            .application_query(repeated_query::definition())
            .application_query_binding::<super::paired::membership::MembershipBinding>()
            .application_query(super::paired::membership::definition())
            .application_query_binding::<super::paired::first_query::FirstBinding>()
            .application_query(super::paired::first_query::definition())
            .application_query_binding::<super::continuation::query::PageBinding>()
            .application_query(super::continuation::query::definition())
            .application_query_binding::<super::live::query::LiveBinding>()
            .application_query(super::live::query::definition())
            .effect(super::live::cause::ValueChanged::reference())
            .operation(
                super::live::publish::PublishValue::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(super::live::publish::PublishValue::reference(), 1)
            .operation_emit(
                super::live::publish::PublishValue::reference(),
                super::live::cause::ValueChanged::reference(),
            )
            .application_mutation_binding::<super::live::publish::PublishBinding>())
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        AdmissionSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DocumentPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}
