//! Installation-owned derived evidence cannot replace a live action grant.
pub use super::super::{
    grants::{GrantGrantee, GrantResource},
    host,
    schema::*,
};
pub use worth_query_decl::facade::application_schema::*;
pub use worth_query_host::facade::{
    application_invariants::EntityId, runtime::ExecutionAllocationPolicy,
};

pub struct GrantIncidence;
impl GrantIncidence {
    pub fn reference() -> ApplicationDerivedRelationRef<CapabilitySchema, Self, Principal, Document>
    {
        ApplicationDerivedRelationRef::from_schema_identifiers(
            "GrantIncidence",
            "Principal",
            "Document",
        )
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    schema.derived_relation(
        GrantIncidence::reference(),
        ApplicationDerivedRelation::relation(GrantGrantee::reference())
            .then(GrantResource::reference()),
        ApplicationRelationGraphLaws::unbounded(),
    )
}

/// This trusted projection is deliberately independent of the actor's permission.
/// The real Resource action still observes GrantStatus through its own policy.
pub fn observed_document(host: &host::Host) -> EntityId {
    let completed = host
        .application
        .runtime()
        .with_host_advancement(|phase| {
            host.projection
                .project(
                    &phase,
                    |reader| {
                        let principal = reader
                            .resolve_entity(PrincipalIdentityField::reference(), 1)
                            .unwrap();
                        let root = reader.entity_member(&principal).unwrap();
                        let derived = reader
                            .derived_member_relations_from(GrantIncidence::reference(), &root)
                            .unwrap();
                        assert_eq!(derived.len(), 1, "real deduplicated derived endpoint count");
                        let document = derived[0]
                            .to()
                            .as_entity()
                            .expect("an actual physical endpoint");
                        assert_eq!(
                            reader
                                .field(document, DocumentIdentityField::reference())
                                .unwrap(),
                            DOCUMENT_IDENTITY
                        );
                        document.entity_id()
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (document, snapshot, _) = completed.into_parts();
    drop(snapshot);
    document
}
