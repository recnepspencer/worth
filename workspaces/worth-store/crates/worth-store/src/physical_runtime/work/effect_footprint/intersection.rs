use super::{
    PhysicalEffectAccess, PhysicalEffectFootprint, PhysicalEffectKey, PhysicalEffectRelation,
};

pub(in crate::physical_runtime) fn effect_relation(
    left: &PhysicalEffectFootprint,
    right: &PhysicalEffectFootprint,
) -> PhysicalEffectRelation {
    if left.store() != right.store() {
        return PhysicalEffectRelation::Disjoint;
    }
    let mut shared_read = false;
    for left_key in left.keys() {
        for right_key in right.keys() {
            match key_relation(*left_key, *right_key) {
                PhysicalEffectRelation::Conflict => return PhysicalEffectRelation::Conflict,
                PhysicalEffectRelation::SharedRead => shared_read = true,
                PhysicalEffectRelation::Disjoint => {}
            }
        }
    }
    if shared_read {
        PhysicalEffectRelation::SharedRead
    } else {
        PhysicalEffectRelation::Disjoint
    }
}

pub(in crate::physical_runtime) fn shares_coordination_identity(
    left: &PhysicalEffectFootprint,
    right: &PhysicalEffectFootprint,
) -> bool {
    left.store() == right.store()
        && left.keys().iter().any(|left_key| {
            right
                .keys()
                .iter()
                .any(|right_key| same_identity(*left_key, *right_key))
        })
}

fn key_relation(left: PhysicalEffectKey, right: PhysicalEffectKey) -> PhysicalEffectRelation {
    use PhysicalEffectKey as Key;
    match (left, right) {
        (
            Key::Range {
                artifact: left_artifact,
                ..
            }
            | Key::WholeArtifact {
                artifact: left_artifact,
                ..
            }
            | Key::DeleteArtifact {
                artifact: left_artifact,
            },
            Key::Range {
                artifact: right_artifact,
                ..
            }
            | Key::WholeArtifact {
                artifact: right_artifact,
                ..
            }
            | Key::DeleteArtifact {
                artifact: right_artifact,
            },
        ) if left_artifact == right_artifact => artifact_relation(left, right),
        (
            Key::Wal {
                segment: left_segment,
                generation: left_generation,
                ..
            }
            | Key::DeleteWal {
                segment: left_segment,
                generation: left_generation,
            },
            Key::Wal {
                segment: right_segment,
                generation: right_generation,
                ..
            }
            | Key::DeleteWal {
                segment: right_segment,
                generation: right_generation,
            },
        ) if left_segment == right_segment && left_generation == right_generation => {
            wal_relation(left, right)
        }
        (
            Key::Allocator {
                generation: left_generation,
                block: left_block,
            },
            Key::Allocator {
                generation: right_generation,
                block: right_block,
            },
        ) if left_generation == right_generation
            && (left_block.is_none() || right_block.is_none() || left_block == right_block) =>
        {
            PhysicalEffectRelation::Conflict
        }
        (Key::RootPublication, Key::RootPublication) | (Key::Namespace, Key::Namespace) => {
            PhysicalEffectRelation::Conflict
        }
        (Key::Checkpoint { .. }, Key::Checkpoint { .. }) => checkpoint_relation(left, right),
        (
            Key::Obligation {
                identity: left_identity,
                access: left_access,
            },
            Key::Obligation {
                identity: right_identity,
                access: right_access,
            },
        ) if left_identity == right_identity => share_or_conflict(left_access, right_access),
        _ => PhysicalEffectRelation::Disjoint,
    }
}

fn artifact_relation(left: PhysicalEffectKey, right: PhysicalEffectKey) -> PhysicalEffectRelation {
    if is_delete(left) || is_delete(right) || contains(left, right) {
        if is_delete(left) || is_delete(right) {
            return PhysicalEffectRelation::Conflict;
        }
        return share_or_conflict(access_of(left), access_of(right));
    }
    PhysicalEffectRelation::Disjoint
}

fn wal_relation(left: PhysicalEffectKey, right: PhysicalEffectKey) -> PhysicalEffectRelation {
    if matches!(left, PhysicalEffectKey::DeleteWal { .. })
        || matches!(right, PhysicalEffectKey::DeleteWal { .. })
        || contains(left, right)
    {
        if matches!(left, PhysicalEffectKey::DeleteWal { .. })
            || matches!(right, PhysicalEffectKey::DeleteWal { .. })
        {
            return PhysicalEffectRelation::Conflict;
        }
        return share_or_conflict(access_of(left), access_of(right));
    }
    PhysicalEffectRelation::Disjoint
}

fn checkpoint_relation(
    left: PhysicalEffectKey,
    right: PhysicalEffectKey,
) -> PhysicalEffectRelation {
    if contains(left, right) {
        share_or_conflict(access_of(left), access_of(right))
    } else {
        PhysicalEffectRelation::Disjoint
    }
}

fn contains(left: PhysicalEffectKey, right: PhysicalEffectKey) -> bool {
    match (interval(left), interval(right)) {
        (Some((left_start, left_end, left_whole)), Some((right_start, right_end, right_whole))) => {
            left_whole || right_whole || (left_start < right_end && right_start < left_end)
        }
        _ => false,
    }
}

fn interval(key: PhysicalEffectKey) -> Option<(u64, u64, bool)> {
    match key {
        PhysicalEffectKey::Range { start, end, .. } => Some((start, end, false)),
        PhysicalEffectKey::WholeArtifact { .. } | PhysicalEffectKey::DeleteArtifact { .. } => {
            Some((0, u64::MAX, true))
        }
        PhysicalEffectKey::Wal { start, end, .. } => Some((start, end, false)),
        PhysicalEffectKey::DeleteWal { .. } => Some((0, u64::MAX, true)),
        PhysicalEffectKey::Checkpoint {
            start, end, whole, ..
        } => Some((start, end, whole)),
        PhysicalEffectKey::Allocator { .. }
        | PhysicalEffectKey::RootPublication
        | PhysicalEffectKey::Namespace
        | PhysicalEffectKey::Obligation { .. } => None,
    }
}

fn is_delete(key: PhysicalEffectKey) -> bool {
    matches!(key, PhysicalEffectKey::DeleteArtifact { .. })
}

fn access_of(key: PhysicalEffectKey) -> PhysicalEffectAccess {
    match key {
        PhysicalEffectKey::Range { access, .. }
        | PhysicalEffectKey::WholeArtifact { access, .. }
        | PhysicalEffectKey::Wal { access, .. }
        | PhysicalEffectKey::Checkpoint { access, .. }
        | PhysicalEffectKey::Obligation { access, .. } => access,
        PhysicalEffectKey::DeleteArtifact { .. }
        | PhysicalEffectKey::DeleteWal { .. }
        | PhysicalEffectKey::Allocator { .. }
        | PhysicalEffectKey::RootPublication
        | PhysicalEffectKey::Namespace => PhysicalEffectAccess::Write,
    }
}

fn share_or_conflict(
    left: PhysicalEffectAccess,
    right: PhysicalEffectAccess,
) -> PhysicalEffectRelation {
    if left == PhysicalEffectAccess::Read && right == PhysicalEffectAccess::Read {
        PhysicalEffectRelation::SharedRead
    } else {
        PhysicalEffectRelation::Conflict
    }
}

fn same_identity(left: PhysicalEffectKey, right: PhysicalEffectKey) -> bool {
    !matches!(key_relation(left, right), PhysicalEffectRelation::Disjoint)
        || matches!(
            (left, right),
            (
                PhysicalEffectKey::Range { artifact: left_artifact, .. }
                | PhysicalEffectKey::WholeArtifact { artifact: left_artifact, .. }
                | PhysicalEffectKey::DeleteArtifact { artifact: left_artifact },
                PhysicalEffectKey::Range { artifact: right_artifact, .. }
                | PhysicalEffectKey::WholeArtifact { artifact: right_artifact, .. }
                | PhysicalEffectKey::DeleteArtifact { artifact: right_artifact },
            ) if left_artifact == right_artifact
        )
        || matches!(
            (left, right),
            (
                PhysicalEffectKey::Wal { segment: left_segment, generation: left_generation, .. }
                | PhysicalEffectKey::DeleteWal { segment: left_segment, generation: left_generation },
                PhysicalEffectKey::Wal { segment: right_segment, generation: right_generation, .. }
                | PhysicalEffectKey::DeleteWal { segment: right_segment, generation: right_generation },
            ) if left_segment == right_segment && left_generation == right_generation
        )
        || matches!(
            (left, right),
            (
                PhysicalEffectKey::Checkpoint { .. },
                PhysicalEffectKey::Checkpoint { .. }
            ) | (PhysicalEffectKey::Namespace, PhysicalEffectKey::Namespace)
                | (
                    PhysicalEffectKey::RootPublication,
                    PhysicalEffectKey::RootPublication
                )
        )
}
