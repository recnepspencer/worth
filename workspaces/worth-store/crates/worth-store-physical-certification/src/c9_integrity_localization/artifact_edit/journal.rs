//! Typed journal edits use immutable production coordinates, never validation results.
use super::{ArtifactGranule, ArtifactInventory, ArtifactOperator as Op, FrameGrammar};
use std::path::Path;
use worth_store::physical_runtime::PhysicalIntegrityScrubTarget;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

mod checkpoint;
mod checkpoint_audit;
mod wal;

pub(in crate::c9_integrity_localization) fn operators(granule: &ArtifactGranule) -> Vec<Op> {
    let mut rows = vec![Op::CoveredByte, Op::Checksum, Op::Length, Op::ScopeSubstitution,
        Op::Truncate, Op::EnvelopeVersion];
    if matches!(granule.family, "checkpoint_dirty_basis" | "checkpoint_binding" | "checkpoint_footer") {
        rows.push(Op::SelectiveAggregate);
    }
    rows
}

pub(in crate::c9_integrity_localization) fn inspection_target(
    target: &ArtifactGranule, operator: Op,
) -> PhysicalIntegrityScrubTarget {
    if target.family == "checkpoint_binding" && operator == Op::ScopeSubstitution {
        let scope = PhysicalArtifactScope::checkpoint_binding(
            target.scope.checkpoint_identity().unwrap(),
            PhysicalByteRange::new(target.offset() as u64, 36).unwrap(),
        );
        PhysicalIntegrityScrubTarget::new(target.target, scope).unwrap()
    } else { target.scrub_target() }
}

pub(super) fn apply(root: &Path, baseline: &Path, inventory: &ArtifactInventory, index: usize, operator: Op) {
    let target = &inventory.granules[index];
    let path = root.join(&target.path);
    let mut bytes = std::fs::read(&path).unwrap();
    if operator == Op::Truncate {
        bytes.truncate(target.offset() + target.length() / 2);
    } else if target.grammar == FrameGrammar::Wal {
        wal::edit(&mut bytes[target.offset()..target.offset()+target.length()], operator);
    } else {
        checkpoint::edit(&mut bytes, baseline, inventory, index, operator);
    }
    std::fs::write(path, bytes).unwrap();
}

pub(super) fn audit(before: &[u8], after: &[u8], target: &ArtifactGranule, operator: Op) {
    if operator == Op::Truncate {
        assert_eq!(after.len(), target.offset() + target.length() / 2);
        assert_eq!(after, &before[..after.len()]);
        return;
    }
    if target.grammar == FrameGrammar::Wal {
        let end = target.offset()+target.length();
        assert_eq!(&before[..target.offset()], &after[..target.offset()]);
        assert_eq!(&before[end..], &after[end..]);
        wal::audit(&before[target.offset()..end], &after[target.offset()..end], operator);
    } else {
        checkpoint_audit::audit(before, after, target, operator);
    }
}
