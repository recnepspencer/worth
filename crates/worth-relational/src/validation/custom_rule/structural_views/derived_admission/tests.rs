//! Exact registered owners remain distinct even when their descriptions agree.
use super::{DerivedAdmissionCache, PROBES};
use crate::derived_relation::*;
use crate::facade::config::CrossContextPolicy;
use crate::graph_member::NativeMemberDomain;
use crate::identity::data::KindId;
use crate::schema::data::{
    EndpointKindContractDeclaration, RelationIntegrityDeclarations, RelationalSchemaRegistry,
    SchemaId, SchemaVersionId,
};
use crate::tests::support::RelationIntegritySchemaFixture;

fn catalog() -> (RelationalSchemaRegistry, Vec<DerivedRelationHandle>) {
    let parts = (0..10)
        .map(|index| DerivedRelationDefinitionParts {
            identity: DerivedRelationIdentity {
                schema: SchemaId("test".into()),
                version: SchemaVersionId(1),
                name: format!("admission-{index}"),
            },
            from: NativeMemberDomain::Entity(KindId(1)),
            to: NativeMemberDomain::Entity(KindId(1)),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(1),
            }],
            laws: None,
        })
        .collect::<Vec<_>>();
    let identities = parts
        .iter()
        .map(|parts| parts.identity.clone())
        .collect::<Vec<_>>();
    let registry = RelationIntegritySchemaFixture {
        relation_integrity: RelationIntegrityDeclarations::new(
            vec![EndpointKindContractDeclaration {
                contract_id: "admission-endpoints".into(),
                allowed_source_kinds: vec![KindId(1)],
                allowed_target_kinds: vec![KindId(1)],
                self_edges_allowed: true,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
            }],
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        ..Default::default()
    }
    .build_registry()
    .register_derived_relations(parts)
    .unwrap();
    let handles = identities
        .iter()
        .map(|id| registry.derived_relation(id).unwrap())
        .collect();
    (registry, handles)
}
#[test]
fn admission_cache_keeps_registered_identity_across_clones_and_full_capacity() {
    let (_, handles) = catalog();
    let (_, foreign) = catalog();
    assert_eq!(handles[0].parts(), foreign[0].parts());
    let cache = DerivedAdmissionCache::default();
    cache.remember(&handles[0]);
    let cloned = cache.clone();
    assert!(cloned.contains(&handles[0]));
    assert!(!cloned.contains(&foreign[0]));
    // A clone is a separate immutable-view admission history.
    cache.remember(&handles[1]);
    assert!(!cloned.contains(&handles[1]));
    for handle in &handles {
        cache.remember(handle);
    }
    assert!(handles[..PROBES as usize]
        .iter()
        .all(|handle| cache.contains(handle)));
    assert!(!cache.contains(handles.last().unwrap()));
    assert!(!DerivedAdmissionCache::default().contains(&handles[0]));
}
