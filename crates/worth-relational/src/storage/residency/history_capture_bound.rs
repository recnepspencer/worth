//! Checkpoint consolidation policy bounds restart-induced history folding.
use crate::durability::data::RelationalHistoryMemberDirectory;
use crate::durability::medium::RelationalDurableMemberLocator;

pub(crate) const FULL_HISTORY_READ_LIMIT: u64 = 1 << 30;
pub(crate) const HISTORY_CAPTURE_MAX_LAYERS: usize = 8;
pub(crate) const HISTORY_CAPTURE_INPUT_TARGET: u64 = 64 << 20;

/// One flat image may exceed the target because of intrinsic record history.
/// Extra dependency layers may not: consolidate before publishing that image.
pub(super) fn needs_consolidation(
    directory: &RelationalHistoryMemberDirectory,
    root: RelationalDurableMemberLocator,
) -> bool {
    let (layers, input_bytes) = directory.fold_span(root);
    layers > HISTORY_CAPTURE_MAX_LAYERS
        || (layers > 1 && input_bytes > HISTORY_CAPTURE_INPUT_TARGET)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durability::data::RelationalCheckpointHistoryDependency;

    #[test]
    fn encoded_input_threshold_counts_shared_bases_once_and_exempts_flat_images() {
        let member = |id, length| RelationalDurableMemberLocator {
            sha256: [id; 32],
            length,
        };
        let base = member(1, HISTORY_CAPTURE_INPUT_TARGET - 1024);
        let left = member(2, 2048);
        let right = member(3, 2048);
        let root = member(4, 2048);
        let directory = RelationalHistoryMemberDirectory::readmit(
            &[
                RelationalCheckpointHistoryDependency {
                    member: left,
                    bases: vec![base],
                },
                RelationalCheckpointHistoryDependency {
                    member: right,
                    bases: vec![base],
                },
                RelationalCheckpointHistoryDependency {
                    member: root,
                    bases: vec![left, right],
                },
            ],
            [root],
        )
        .unwrap();
        assert_eq!(
            directory.fold_span(root),
            (3, base.length + left.length + right.length + root.length)
        );
        assert!(needs_consolidation(&directory, root));
        let flat = RelationalHistoryMemberDirectory::default();
        assert!(!needs_consolidation(
            &flat,
            member(5, HISTORY_CAPTURE_INPUT_TARGET * 2)
        ));
        assert!(!needs_consolidation(&flat, member(6, 1024)));
    }
}
