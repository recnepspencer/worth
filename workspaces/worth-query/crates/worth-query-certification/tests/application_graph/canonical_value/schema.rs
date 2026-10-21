//! One independently installed source field using the generated canonical binding.
use super::{binding::NestedBinding, model::*, query};
use declaration::{application_schema::*, authentication::*};
use worth_query_host::facade::{declaration, *};

worth_query_application! {
    pub CanonicalSchema {
        owner: "canonical_nested_value", version: (1, 0),
        contributions: [CanonicalContribution],
    }
}
worth_query_application_contribution! {
    pub contribution CanonicalContribution in CanonicalSchema {
        identity: "canonical-nested.application.v1",
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
        }
    }
}
worth_query_entity!(pub ExternalMapping for CanonicalSchema);
worth_query_entity!(pub Principal for CanonicalSchema);
worth_query_entity!(pub Document for CanonicalSchema);
worth_query_aspect!(pub ExternalIdentity for CanonicalSchema, ExternalMapping; identity = AspectIdentity(0x91768001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for CanonicalSchema, Principal; identity = AspectIdentity(0x91768002), revision = AspectContractRevision(1));
worth_query_aspect!(pub DocumentFacts for CanonicalSchema, Document; identity = AspectIdentity(0x91768003), revision = AspectContractRevision(1));
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
