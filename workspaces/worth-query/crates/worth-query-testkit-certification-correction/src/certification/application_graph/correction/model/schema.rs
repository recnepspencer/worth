//! Authored fields and the installed correction association.
#[path = "schema/records.rs"]
pub(super) mod storage_model;
pub use super::domain_binding::{Counter, InlineCounterBinding};
pub use super::write_admission::{
    BinaryCounter, BrokenBinaryBinding, CanonicalBinding, CanonicalCounter, DelegatedBinding,
    DelegatedCounter,
};
use super::{correction::RecordedEdit, other_edit, principal_status, query};
pub use storage_model::{
    Calculated, Ceiling, ExternalIdentity, ExternalIdentityField, ExternalMapping, Key,
    MappingStatusField, MappingTarget, Number, NumberFacts, NumericModel, NumericModelBindings,
    Operand, Primary, Principal, PrincipalFacts, PrincipalIdentityField, ReadEnabled, Secondary,
};
use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryDisclosureContract, ApplicationQueryInspectionDefinition,
};
use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    *,
};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct NumericSchema;
impl Fragment for NumericSchema {
    const NAME: &'static str = "NumericSchema";
    type Bindings = NoBindings;
    type Members = (NumericContribution,);
}
impl ApplicationSchema for NumericSchema {
    const OWNER: &'static str = "numeric_correction";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 2;
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
impl NumericSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for NumericSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct NumericContribution;
impl ApplicationSchemaContribution<NumericSchema> for NumericContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("numeric-correction.application.v1");
    type Members = (
        NumericModel,
        Contracts,
        (
            RecordedEdit,
            super::compensation_movement::NeutralizeCredit,
            super::source_output_correction::RecordIssued,
            super::composite_definition_model::CorrectDefinition,
            super::authored_identity_correction::UndoCreate,
            super::authored_identity_correction::UndoRetire,
            super::adjacency_link::ReverseLink,
            super::observed_clone::UndoClone,
        ),
    );
}
impl NumericContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<NumericSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<NumericSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<NumericSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = super::causal_inspection::declare(schema);
            let schema = super::operand_edit::declare(schema);
            let schema = super::write_admission::declare(schema);
            let schema = super::aspect_input::declare(schema);
            let schema = super::scalar_input::declare(schema);
            let schema = super::composite_definition_model::declare(schema);
            let schema = super::authored_identity_model::declare(schema);
            let schema = super::authored_identity_reference::declare(schema);
            let schema = super::adjacency_link::declare(schema);
            super::source_output_model::declare(super::compensation_movement::declare(
                super::calculation::declare(super::adoption::declare(schema)),
            ))
            .principal_binding(ActorBinding::reference())
            .application_query(query::definition())
            .application_query(query::full::definition())
            .application_query_binding::<query::full::Binding>()
            .application_query_inspection(
                ApplicationQueryInspectionDefinition::<NumericSchema, query::full::Binding>::new(
                    ApplicationQueryDisclosureContract::public(),
                )
                .unwrap(),
            )
            .application_query(super::calculation_query::definition())
            .operation(
                other_edit::EditOther::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(other_edit::EditOther::reference(), 8)
            .operation_read_field(other_edit::EditOther::reference(), Key::at("Key"))
            .operation_read_field(other_edit::EditOther::reference(), Primary::at("Primary"))
            .operation_read_field(
                other_edit::EditOther::reference(),
                Secondary::at("Secondary"),
            )
            .operation_read_field(
                other_edit::EditOther::reference(),
                ReadEnabled::at("ReadEnabled"),
            )
            .operation_read_field(other_edit::EditOther::reference(), Ceiling::at("Ceiling"))
            .operation_write(
                other_edit::EditOther::reference(),
                Secondary::at("Secondary"),
            )
            .operation_write(other_edit::EditOther::reference(), Primary::at("Primary"))
            .operation_write(
                other_edit::EditOther::reference(),
                ReadEnabled::at("ReadEnabled"),
            )
            .operation_write(other_edit::EditOther::reference(), Ceiling::at("Ceiling"))
            .application_mutation_binding::<other_edit::OtherBinding>()
            .operation(
                principal_status::Disable::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(principal_status::Disable::reference(), 8)
            .operation_read_field(
                principal_status::Disable::reference(),
                ExternalIdentityField::at("ExternalIdentityField"),
            )
            .operation_read_field(
                principal_status::Disable::reference(),
                MappingStatusField::at("MappingStatusField"),
            )
            .operation_write(
                principal_status::Disable::reference(),
                MappingStatusField::at("MappingStatusField"),
            )
            .application_mutation_binding::<principal_status::Binding>()
        })
    }
}

pub struct ActorBinding;
impl ActorBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        NumericSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "ActorBinding",
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
