//! Install the grant used by recovery and adoption histories.
use super::super::super::entry_correction::capability::*;
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;
pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>, scope: &str) {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Seed,
        WorthQueryApplicationRelationSeed as Relation,
    };
    fn key<Entity>(s: &str) -> Key<CheckpointSchema, Entity> {
        Key::new(s).unwrap()
    }
    graph
        .bind_entity(
            Seed::new(CorrectionGrant::reference(), key("entry-correction-grant"))
                .field(CorrectionAction::reference(), 1)
                .field(CorrectionPurpose::reference(), 1)
                .field(CorrectionStatus::reference(), 1)
                .field(CorrectionScopeKey::reference(), scope.to_owned())
                .field(CorrectionNotBefore::reference(), 0)
                .field(CorrectionNotAfter::reference(), u64::MAX)
                .field(CorrectionDepth::reference(), 0),
        )
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionResource::reference(),
            "correction-resource",
            key("entry-correction-grant"),
            key(scope),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantee::reference(),
            "correction-grantee",
            key("model-owner"),
            key("entry-correction-grant"),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantor::reference(),
            "correction-grantor",
            key("model-owner"),
            key("entry-correction-grant"),
        ))
        .unwrap();
}
