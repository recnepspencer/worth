//! Native immutable entity classification derived from the authored signing declarations.
use super::{
    required_kind, WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphLayout,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCreation as Creation,
    ApplicationSigningProvenanceEncoding as Encoding,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;

#[derive(Debug)]
pub(super) struct SignedEntityLayout {
    kinds: BTreeSet<KindId>,
    encodings: BTreeMap<Locus, Encoding>,
    creations: BTreeMap<String, (String, Creation)>,
}

pub(super) fn lower(
    schema: &ErasedApplicationSchemaDeclaration,
    kinds: &BTreeMap<String, KindId>,
) -> Result<SignedEntityLayout, WorthQueryPrimaryGraphInstallationDenial> {
    let mut signed = SignedEntityLayout {
        kinds: BTreeSet::new(),
        encodings: BTreeMap::new(),
        creations: BTreeMap::new(),
    };
    for member in schema.members() {
        let ApplicationSchemaMember::OperationBody { operation, body } = member else {
            continue;
        };
        let Some(creation) = body
            .creation()
            .filter(|creation| creation.signing().is_some())
        else {
            continue;
        };
        let signing = creation.signing().expect("filtered native signed creation");
        signed
            .kinds
            .insert(required_kind(kinds, creation.entity())?);
        for field in creation.fields() {
            if field.field() == signing.provenance() {
                let encoding = Encoding::for_binding(field.binding())
                    .expect("freshly admitted native provenance codec");
                signed.encodings.insert(field.field().clone(), encoding);
            }
        }
        signed.creations.insert(
            operation.clone(),
            (
                body.scope_entity().expect("admitted signing scope").into(),
                creation.clone(),
            ),
        );
    }
    Ok(signed)
}
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn has_signed_entity_kinds(&self) -> bool {
        !self.signing.kinds.is_empty()
    }
    pub(in crate::domain_computation::primary_graph) fn is_signed_entity_kind(
        &self,
        kind: KindId,
    ) -> bool {
        self.signing.kinds.contains(&kind)
    }
    pub(in crate::domain_computation::primary_graph) fn signing_provenance_encoding(
        &self,
        locus: &Locus,
    ) -> Option<Encoding> {
        self.signing.encodings.get(locus).copied()
    }
    pub(in crate::domain_computation::primary_graph) fn signed_creation_contract(
        &self,
        operation: &str,
    ) -> Option<(&str, &Creation)> {
        self.signing
            .creations
            .get(operation)
            .map(|(scope, creation)| (scope.as_str(), creation))
    }
}
