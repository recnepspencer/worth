//! Authored root/child values and the counter changed only after admitted reads.
use super::{
    binding::NestedBinding, opaque::SwallowingValue, operation, ordinary_attempt, query,
    repeated_query,
};
use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    *,
};

worth_query_application! { pub AdmissionSchema {
    owner: "canonical_admission", version: (1, 0), contributions: [AdmissionContribution],
}}
worth_query_application_contribution! { pub contribution AdmissionContribution in AdmissionSchema {
    identity: "canonical-admission.application.v1",
    members: |schema| {
        schema.entity(ExternalMapping::reference()).entity(Principal::reference()).entity(Document::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference()).aspect(Document::reference(), DocumentFacts::reference())
            .field(ExternalMapping::reference(), ExternalIdentityField::reference()).field(ExternalMapping::reference(), MappingStatusField::reference())
            .field(Principal::reference(), PrincipalIdentityField::reference())
            .field(Document::reference(), DocumentIdentityField::reference())
            .field(Document::reference(), DocumentValueField::reference()).field(Document::reference(), CounterField::reference())
            .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
            .relation(ChildLink::reference(), Document::reference(), Document::reference())
            .principal_binding(DocumentPrincipalBinding::reference())
            .operation(operation::Advance::reference().definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(operation::Advance::reference(), 32)
            .operation_read_field(operation::Advance::reference(), DocumentIdentityField::reference())
            .operation_read_field(operation::Advance::reference(), DocumentValueField::reference())
            .operation_read_field(operation::Advance::reference(), CounterField::reference())
            .operation_read_relation(operation::Advance::reference(), ChildLink::reference())
            .operation_write(operation::Advance::reference(), CounterField::reference())
            .application_mutation_binding::<operation::AdvanceBinding>()
            .operation(ordinary_attempt::ObserveCanonical::reference().definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(ordinary_attempt::ObserveCanonical::reference(), 2)
            .operation_read_field(ordinary_attempt::ObserveCanonical::reference(), DocumentValueField::reference())
            .application_query_binding::<query::ReadBinding>().application_query(query::definition())
            .application_query_binding::<repeated_query::RepeatedBinding>().application_query(repeated_query::definition())
            .application_query_binding::<super::paired::membership::MembershipBinding>()
            .application_query(super::paired::membership::definition())
            .application_query_binding::<super::paired::first_query::FirstBinding>()
            .application_query(super::paired::first_query::definition())
            .application_query_binding::<super::continuation::query::PageBinding>()
            .application_query(super::continuation::query::definition())
            .application_query_binding::<super::live::query::LiveBinding>()
            .application_query(super::live::query::definition())
            .effect(super::live::cause::ValueChanged::reference())
            .operation(super::live::publish::PublishValue::reference().definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(super::live::publish::PublishValue::reference(), 1)
            .operation_emit(super::live::publish::PublishValue::reference(), super::live::cause::ValueChanged::reference())
            .application_mutation_binding::<super::live::publish::PublishBinding>()
    }
}}
worth_query_entity!(pub ExternalMapping for AdmissionSchema);
worth_query_entity!(pub Principal for AdmissionSchema);
worth_query_entity!(pub Document for AdmissionSchema);
worth_query_aspect!(pub ExternalIdentity for AdmissionSchema, ExternalMapping; identity = AspectIdentity(0x91768101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for AdmissionSchema, Principal; identity = AspectIdentity(0x91768102), revision = AspectContractRevision(1));
worth_query_aspect!(pub DocumentFacts for AdmissionSchema, Document; identity = AspectIdentity(0x91768103), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for AdmissionSchema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for AdmissionSchema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for AdmissionSchema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for AdmissionSchema, Document, DocumentFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentValueField for AdmissionSchema, Document, DocumentFacts: SwallowingValue => NestedBinding, read_write, equality);
worth_query_field!(pub CounterField for AdmissionSchema, Document, DocumentFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in AdmissionSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ChildLink in AdmissionSchema, Document => Document; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub DocumentPrincipalBinding in AdmissionSchema, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatusField,
    target: MappingTarget => Principal, principal_identity: PrincipalIdentityField
});
