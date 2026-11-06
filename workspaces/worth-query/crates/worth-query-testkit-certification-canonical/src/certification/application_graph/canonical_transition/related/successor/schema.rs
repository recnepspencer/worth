//! One independently installed source field using the generated canonical binding.
#[path = "schema/records.rs"]
mod storage_model;
pub use super::companion::{producer as cp, schema::*};
pub use super::{binding::NestedBinding, derived_query, edit, model::*, output, producer, query};
pub use declaration::authentication::*;
pub use storage_model::*;
pub use worth_query_host::facade::{declaration, *};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct CanonicalSchema;
impl Fragment for CanonicalSchema {
    const NAME: &'static str = "CanonicalSchema";
    type Bindings = NoBindings;
    type Members = (CanonicalContribution,);
}
impl ApplicationSchema for CanonicalSchema {
    const OWNER: &'static str = "canonical_transition_related";
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
impl CanonicalSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for CanonicalSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct CanonicalContribution;
impl ApplicationSchemaContribution<CanonicalSchema> for CanonicalContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("canonical-transition.application.v1");
    type Members = (
        storage_model::StorageModel,
        super::companion::schema::CompanionModel,
        Contracts,
    );
}
impl CanonicalContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<CanonicalSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<CanonicalSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<CanonicalSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CanonicalSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(DocumentPrincipalBinding::reference())
            .application_query_binding::<query::ReadBinding>()
            .application_query(query::definition())
            .application_query_binding::<derived_query::ReadBinding>()
            .application_query(derived_query::definition())
            .operation(
                producer::Produce::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(producer::Produce::reference(), 16)
            .operation_read_field(
                producer::Produce::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                producer::Produce::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_create(producer::Produce::reference(), Derived::reference())
            .operation_write(
                producer::Produce::reference(),
                DerivedIdentityField::at("DerivedIdentityField"),
            )
            .operation_write(
                producer::Produce::reference(),
                DerivedValueField::at("DerivedValueField"),
            )
            .operation_link(producer::Produce::reference(), DerivedChildren::reference())
            .application_mutation_binding::<producer::ProduceBinding>()
            .operation(
                producer::preserve::Preserve::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(producer::preserve::Preserve::reference(), 32)
            .operation_read_field(
                producer::preserve::Preserve::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                producer::preserve::Preserve::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_read_entity(
                producer::preserve::Preserve::reference(),
                Derived::reference(),
            )
            .operation_read_field(
                producer::preserve::Preserve::reference(),
                DerivedValueField::at("DerivedValueField"),
            )
            .operation_write(
                producer::preserve::Preserve::reference(),
                DerivedValueField::at("DerivedValueField"),
            )
            .application_mutation_binding::<producer::preserve::Binding>()
            .operation(
                edit::Edit::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(edit::Edit::reference(), 16)
            .operation_read_field(
                edit::Edit::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                edit::Edit::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_write(
                edit::Edit::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .application_mutation_binding::<edit::EditBinding>()
            .application_query_binding::<super::page_query::PageBinding>()
            .application_query(super::page_query::definition())
            .application_query_binding::<super::live_query::LiveBinding>()
            .application_query(super::live_query::definition())
            .application_query_binding::<super::membership::MembershipBinding>()
            .application_query(super::membership::definition())
            .application_query_binding::<super::paired_query::ReadBinding>()
            .application_query(super::paired_query::definition())
            .effect(super::cause::ValueChanged::reference())
            .operation(
                super::publish::PublishValue::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(super::publish::PublishValue::reference(), 8)
            .operation_emit(
                super::publish::PublishValue::reference(),
                super::cause::ValueChanged::reference(),
            )
            .application_mutation_binding::<super::publish::PublishBinding>()
            .operation(
                super::retire::Retire::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(super::retire::Retire::reference(), 64)
            .operation_read_entity(super::retire::Retire::reference(), Derived::reference())
            .operation_read_field(
                super::retire::Retire::reference(),
                DerivedValueField::at("DerivedValueField"),
            )
            .operation_read_relation(
                super::retire::Retire::reference(),
                DerivedChildren::reference(),
            )
            .operation_unlink(
                super::retire::Retire::reference(),
                DerivedChildren::reference(),
            )
            .operation_delete(super::retire::Retire::reference(), Derived::reference())
            .application_mutation_binding::<super::retire::Binding>()
            .invariant(super::companion::rule::definition())
            .application_query_binding::<super::companion::read::ReadBinding>()
            .application_query(super::companion::read::definition())
            .operation(
                cp::CreateCompanion::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(cp::CreateCompanion::reference(), 32)
            .operation_read_field(
                cp::CreateCompanion::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                cp::CreateCompanion::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_create(cp::CreateCompanion::reference(), Companion::reference())
            .operation_create(cp::CreateCompanion::reference(), Derived::reference())
            .operation_write(
                cp::CreateCompanion::reference(),
                DerivedIdentityField::at("DerivedIdentityField"),
            )
            .operation_write(
                cp::CreateCompanion::reference(),
                DerivedValueField::at("DerivedValueField"),
            )
            .operation_write(
                cp::CreateCompanion::reference(),
                CompanionIdentityField::at("CompanionIdentityField"),
            )
            .operation_write(
                cp::CreateCompanion::reference(),
                CompanionValueField::at("CompanionValueField"),
            )
            .operation_link(cp::CreateCompanion::reference(), CompanionLink::reference())
            .application_mutation_binding::<cp::ProduceBinding>()
            .operation(
                cp::preserve::PreserveCompanion::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(cp::preserve::PreserveCompanion::reference(), 32)
            .operation_read_field(
                cp::preserve::PreserveCompanion::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .operation_read_field(
                cp::preserve::PreserveCompanion::reference(),
                DocumentValueField::at("DocumentValueField"),
            )
            .operation_read_entity(
                cp::preserve::PreserveCompanion::reference(),
                Companion::reference(),
            )
            .operation_read_field(
                cp::preserve::PreserveCompanion::reference(),
                CompanionValueField::at("CompanionValueField"),
            )
            .operation_write(
                cp::preserve::PreserveCompanion::reference(),
                CompanionValueField::at("CompanionValueField"),
            )
            .application_mutation_binding::<cp::preserve::Binding>())
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        CanonicalSchema,
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
