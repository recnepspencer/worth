//! One independently installed source field using the generated canonical binding.
use super::{binding::NestedBinding, derived_query, edit, model::*, output, producer, query};
use declaration::{application_schema::*, authentication::*};
use worth_query_host::facade::{declaration, *};

worth_query_application! {
    pub CanonicalSchema {
        owner: "canonical_transition", version: (1, 0),
        contributions: [CanonicalContribution],
    }
}
worth_query_application_contribution! {
    pub contribution CanonicalContribution in CanonicalSchema {
        identity: "canonical-transition.application.v1",
        members: |schema| {
            schema.entity(ExternalMapping::reference()).entity(Principal::reference()).entity(Document::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference()).aspect(Document::reference(), DocumentFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference()).field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Document::reference(), DocumentIdentityField::reference()).field(Document::reference(), DocumentValueField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(DocumentPrincipalBinding::reference())
                .application_query_binding::<query::ReadBinding>()
                .application_query(query::definition())
                .entity(Derived::reference()).aspect(Derived::reference(), DerivedFacts::reference())
                .field(Derived::reference(), DerivedIdentityField::reference()).field(Derived::reference(), DerivedValueField::reference())
                .application_query_binding::<derived_query::ReadBinding>().application_query(derived_query::definition())
                .operation(producer::Produce::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(producer::Produce::reference(), 16)
                .operation_read_field(producer::Produce::reference(), DocumentIdentityField::reference())
                .operation_read_field(producer::Produce::reference(), DocumentValueField::reference())
                .operation_create(producer::Produce::reference(), Derived::reference())
                .operation_write(producer::Produce::reference(), DerivedIdentityField::reference())
                .operation_write(producer::Produce::reference(), DerivedValueField::reference())
                .application_mutation_binding::<producer::ProduceBinding>()
                .operation(producer::preserve::Preserve::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(producer::preserve::Preserve::reference(), 32)
                .operation_read_field(producer::preserve::Preserve::reference(), DocumentIdentityField::reference())
                .operation_read_field(producer::preserve::Preserve::reference(), DocumentValueField::reference())
                .operation_read_entity(producer::preserve::Preserve::reference(), Derived::reference())
                .operation_read_field(producer::preserve::Preserve::reference(), DerivedValueField::reference())
                .operation_write(producer::preserve::Preserve::reference(), DerivedValueField::reference())
                .application_mutation_binding::<producer::preserve::Binding>()
                .operation(edit::Edit::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(edit::Edit::reference(), 16)
                .operation_read_field(edit::Edit::reference(), DocumentIdentityField::reference())
                .operation_read_field(edit::Edit::reference(), DocumentValueField::reference())
                .operation_write(edit::Edit::reference(), DocumentValueField::reference())
                .application_mutation_binding::<edit::EditBinding>()
        }
    }
}
worth_query_entity!(pub ExternalMapping for CanonicalSchema);
worth_query_entity!(pub Principal for CanonicalSchema);
worth_query_entity!(pub Document for CanonicalSchema);
worth_query_aspect!(pub ExternalIdentity for CanonicalSchema, ExternalMapping; identity = AspectIdentity(0x91768201), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for CanonicalSchema, Principal; identity = AspectIdentity(0x91768202), revision = AspectContractRevision(1));
worth_query_aspect!(pub DocumentFacts for CanonicalSchema, Document; identity = AspectIdentity(0x91768203), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for CanonicalSchema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for CanonicalSchema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for CanonicalSchema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for CanonicalSchema, Document, DocumentFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentValueField for CanonicalSchema, Document, DocumentFacts: NestedValue => NestedBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in CanonicalSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub DocumentPrincipalBinding in CanonicalSchema, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatusField,
    target: MappingTarget => Principal, principal_identity: PrincipalIdentityField
});

worth_query_entity!(pub Derived for CanonicalSchema);
worth_query_aspect!(pub DerivedFacts for CanonicalSchema, Derived; identity = AspectIdentity(0x91768204), revision = AspectContractRevision(1));
worth_query_field!(pub DerivedIdentityField for CanonicalSchema, Derived, DerivedFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DerivedValueField for CanonicalSchema, Derived, DerivedFacts: output::DerivedV1 => output::DerivedBinding, read_write, equality);
