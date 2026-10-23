//! Authored fields and the installed correction association.
use super::{correction::RecordedEdit, other_edit, principal_status, query};
use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    *,
};

worth_query_application! {
    pub NumericSchema {
        owner: "numeric_correction", version: (1, 0),
        contributions: [NumericContribution],
    }
}
worth_query_application_contribution! {
    pub contribution NumericContribution in NumericSchema {
        identity: "numeric-correction.application.v1",
        corrections: [RecordedEdit, super::compensation_movement::NeutralizeCredit, super::source_output_correction::RecordIssued, super::composite_definition_model::CorrectDefinition, super::authored_identity_correction::UndoCreate, super::authored_identity_correction::UndoRetire],
        members: |schema| {
            let schema = super::composite_definition_model::declare(schema);
            let schema = super::authored_identity_model::declare(schema);
            let schema = super::authored_identity_reference::declare(schema);
            super::source_output_model::declare(super::compensation_movement::declare(super::calculation::declare(super::adoption::declare(schema)))).entity(ExternalMapping::reference()).entity(Principal::reference()).entity(Number::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference()).aspect(Number::reference(), NumberFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference()).field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Number::reference(), Key::reference()).field(Number::reference(), Primary::reference())
                .field(Number::reference(), Secondary::reference()).field(Number::reference(), ReadEnabled::reference())
                .field(Number::reference(), Ceiling::reference())
                .field(Number::reference(), Calculated::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(ActorBinding::reference())
                .application_query(query::definition())
                .application_query(super::calculation_query::definition())
                .application_query_binding::<super::calculation_query::Binding>()
                .operation(other_edit::EditOther::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(other_edit::EditOther::reference(), 8)
                .operation_read_field(other_edit::EditOther::reference(), Key::reference())
                .operation_read_field(other_edit::EditOther::reference(), Primary::reference())
                .operation_read_field(other_edit::EditOther::reference(), Secondary::reference())
                .operation_read_field(other_edit::EditOther::reference(), ReadEnabled::reference())
                .operation_read_field(other_edit::EditOther::reference(), Ceiling::reference())
                .operation_write(other_edit::EditOther::reference(), Secondary::reference())
                .operation_write(other_edit::EditOther::reference(), Primary::reference())
                .operation_write(other_edit::EditOther::reference(), ReadEnabled::reference())
                .operation_write(other_edit::EditOther::reference(), Ceiling::reference())
                .application_mutation_binding::<other_edit::OtherBinding>()
                .operation(principal_status::Disable::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(principal_status::Disable::reference(), 8)
                .operation_read_field(principal_status::Disable::reference(), ExternalIdentityField::reference())
                .operation_read_field(principal_status::Disable::reference(), MappingStatusField::reference())
                .operation_write(principal_status::Disable::reference(), MappingStatusField::reference())
                .application_mutation_binding::<principal_status::Binding>()
        }
    }
}
worth_query_entity!(pub ExternalMapping for NumericSchema);
worth_query_entity!(pub Principal for NumericSchema);
worth_query_entity!(pub Number for NumericSchema);
worth_query_aspect!(pub ExternalIdentity for NumericSchema, ExternalMapping; identity = AspectIdentity(0x91800101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for NumericSchema, Principal; identity = AspectIdentity(0x91800102), revision = AspectContractRevision(1));
worth_query_aspect!(pub NumberFacts for NumericSchema, Number; identity = AspectIdentity(0x91800103), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for NumericSchema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for NumericSchema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for NumericSchema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Key for NumericSchema, Number, NumberFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Primary for NumericSchema, Number, NumberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Secondary for NumericSchema, Number, NumberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub ReadEnabled for NumericSchema, Number, NumberFacts: bool => BoolApplicationValueBinding, read_write, equality);
worth_query_field!(pub Ceiling for NumericSchema, Number, NumberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Calculated for NumericSchema, Number, NumberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in NumericSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub ActorBinding in NumericSchema, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatusField,
    target: MappingTarget => Principal, principal_identity: PrincipalIdentityField
});
