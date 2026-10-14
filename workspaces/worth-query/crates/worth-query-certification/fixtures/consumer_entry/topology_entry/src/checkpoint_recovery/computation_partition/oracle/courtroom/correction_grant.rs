//! Install the grant used by recovery and adoption histories.
use super::super::super::entry_correction::capability::*;
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;
pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>, scope: &str) {
    seed_grant(graph, scope, "entry-correction-grant", 1);
    seed_grant(graph, scope, "program-adoption-grant", 2);
}
fn seed_grant(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
    scope: &str,
    grant: &str,
    action: u64,
) {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Seed,
        WorthQueryApplicationRelationSeed as Relation,
    };
    fn key<Entity>(s: &str) -> Key<CheckpointSchema, Entity> {
        Key::new(s).unwrap()
    }
    graph
        .bind_entity(
            Seed::new(CorrectionGrant::reference(), key(grant))
                .field(CorrectionAction::reference(), action)
                .field(CorrectionPurpose::reference(), action)
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
            format!("{grant}-resource"),
            key(grant),
            key(scope),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantee::reference(),
            format!("{grant}-grantee"),
            key("model-owner"),
            key(grant),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantor::reference(),
            format!("{grant}-grantor"),
            key("model-owner"),
            key(grant),
        ))
        .unwrap();
}
