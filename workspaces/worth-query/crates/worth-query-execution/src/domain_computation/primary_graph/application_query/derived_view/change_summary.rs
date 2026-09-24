//! Bounded Query dependency changes from the sealed Relational candidate.

use worth_relational::facade::{
    mvcc::PreparedRelationalChangeSummary, publication::RecordStructuralChange,
    transactions::RecordRef,
};

use super::ViewChange;

pub(in crate::domain_computation::primary_graph) fn changes_from_summary(
    summary: &PreparedRelationalChangeSummary,
    maximum_changes: usize,
) -> Option<Vec<ViewChange>> {
    let mut required = 0usize;
    for record in &summary.records {
        // An incomplete summary cannot prove that no retained dependency
        // changed. Publication will make views cold in this case.
        if matches!(record.target, RecordRef::Entity(_))
            && record.structural_change == RecordStructuralChange::Updated
            && record.aspect_scopes.is_empty()
        {
            return None;
        }
        if matches!(record.target, RecordRef::Relation(_))
            && record.before_endpoints.is_none()
            && record.after_endpoints.is_none()
        {
            return None;
        }
        let additional = match record.target {
            RecordRef::Entity(_) => record.aspect_scopes.len().checked_add(1)?,
            RecordRef::Relation(_) => 4,
        };
        required = required.checked_add(additional)?;
        if required > maximum_changes {
            return None;
        }
    }
    let mut changes = Vec::with_capacity(required);
    for record in &summary.records {
        match record.target {
            RecordRef::Entity(entity) => {
                if record.structural_change != RecordStructuralChange::Updated {
                    changes.push(ViewChange::Entity(entity));
                }
                for scope in &record.aspect_scopes {
                    changes.push(match &scope.field_path {
                        Some(path) => {
                            ViewChange::Field(entity, scope.aspect_key.clone(), path.clone())
                        }
                        None => ViewChange::Aspect(entity, scope.aspect_key.clone()),
                    });
                }
            }
            RecordRef::Relation(_) => {
                for endpoints in [record.before_endpoints, record.after_endpoints]
                    .into_iter()
                    .flatten()
                {
                    changes.push(ViewChange::Adjacency(
                        endpoints.source,
                        endpoints.kind_id,
                        0,
                    ));
                    changes.push(ViewChange::Adjacency(
                        endpoints.target,
                        endpoints.kind_id,
                        1,
                    ));
                }
            }
        }
    }
    Some(changes)
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
    use worth_relational::facade::{
        history::{BranchId, CommitId},
        identity::{EntityId, KindId, PartitionId, RelationId, VersionId},
        mvcc::{
            PreparedRelationalAspectScope, PreparedRelationalChangeSummary,
            PreparedRelationalRecordChange, PreparedRelationalRelationEndpoints,
        },
        publication::RecordStructuralChange,
        transactions::RecordRef,
    };

    use super::{changes_from_summary, ViewChange};

    fn entity(slot: u64) -> EntityId {
        EntityId::new(PartitionId::main(), slot, 1)
    }

    fn summary(records: Vec<PreparedRelationalRecordChange>) -> PreparedRelationalChangeSummary {
        PreparedRelationalChangeSummary {
            runtime_instance_id: 1,
            branch_id: BranchId("main".into()),
            before_commit_id: None,
            after_commit_id: CommitId(1),
            before_version: VersionId(0),
            after_version: VersionId(1),
            before_root_identity: 1,
            after_root_identity: 2,
            records,
        }
    }

    #[test]
    fn exact_field_whole_aspect_and_structural_scopes_remain_distinct() {
        let aspect = AspectKey::new("frame").unwrap();
        let path = CanonicalFieldPath::single(FieldKey::new("width").unwrap());
        let summary = summary(vec![
            PreparedRelationalRecordChange {
                target: RecordRef::Entity(entity(1)),
                structural_change: RecordStructuralChange::Updated,
                aspect_scopes: vec![PreparedRelationalAspectScope {
                    aspect_key: aspect.clone(),
                    field_path: Some(path.clone()),
                }],
                before_endpoints: None,
                after_endpoints: None,
            },
            PreparedRelationalRecordChange {
                target: RecordRef::Entity(entity(2)),
                structural_change: RecordStructuralChange::Created,
                aspect_scopes: vec![PreparedRelationalAspectScope {
                    aspect_key: aspect.clone(),
                    field_path: None,
                }],
                before_endpoints: None,
                after_endpoints: None,
            },
        ]);
        let changes = changes_from_summary(&summary, 4).unwrap();
        assert_eq!(changes.len(), 3);
        assert_eq!(
            changes[0],
            ViewChange::Field(entity(1), aspect.clone(), path)
        );
        assert_eq!(changes[1], ViewChange::Entity(entity(2)));
        assert_eq!(changes[2], ViewChange::Aspect(entity(2), aspect));
        assert!(changes_from_summary(&summary, 2).is_none());
    }

    #[test]
    fn moved_relation_dirties_old_and_new_bidirectional_adjacency() {
        let kind_id = KindId::new(7);
        let old = PreparedRelationalRelationEndpoints {
            kind_id,
            source: entity(1),
            target: entity(2),
        };
        let new = PreparedRelationalRelationEndpoints {
            kind_id,
            source: entity(3),
            target: entity(4),
        };
        let summary = summary(vec![PreparedRelationalRecordChange {
            target: RecordRef::Relation(RelationId::new(PartitionId::main(), 1, 1)),
            structural_change: RecordStructuralChange::Updated,
            aspect_scopes: vec![],
            before_endpoints: Some(old),
            after_endpoints: Some(new),
        }]);
        assert_eq!(
            changes_from_summary(&summary, 4).unwrap(),
            vec![
                ViewChange::Adjacency(entity(1), kind_id, 0),
                ViewChange::Adjacency(entity(2), kind_id, 1),
                ViewChange::Adjacency(entity(3), kind_id, 0),
                ViewChange::Adjacency(entity(4), kind_id, 1),
            ]
        );
        assert!(changes_from_summary(&summary, 3).is_none());
    }

    #[test]
    fn missing_changed_scopes_cannot_keep_a_view_warm() {
        for target in [
            RecordRef::Entity(entity(1)),
            RecordRef::Relation(RelationId::new(PartitionId::main(), 1, 1)),
        ] {
            let incomplete = summary(vec![PreparedRelationalRecordChange {
                target,
                structural_change: RecordStructuralChange::Updated,
                aspect_scopes: vec![],
                before_endpoints: None,
                after_endpoints: None,
            }]);
            assert!(changes_from_summary(&incomplete, 4).is_none());
        }
    }
}
