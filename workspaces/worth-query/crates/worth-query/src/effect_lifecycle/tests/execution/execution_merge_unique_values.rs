//! A merge refused by its unique values surfaces a typed denial kind, never
//! a commit failure carrying a debug string.

use worth_query_execution::facade::integration::WorthQueryMergeUniqueValueDenialKind;

use crate::effect_lifecycle::execution_merge::merge_unique_value_denial_kind;
use crate::effect_lifecycle::EffectExecutionDenialKind;

#[test]
fn merge_unique_value_refusals_keep_their_kind() {
    for (refusal, kind, name) in [
        (
            WorthQueryMergeUniqueValueDenialKind::ValueTaken,
            EffectExecutionDenialKind::UniqueValueTaken,
            "unique_value_taken",
        ),
        (
            WorthQueryMergeUniqueValueDenialKind::IndexUnavailable,
            EffectExecutionDenialKind::UniqueIndexUnavailable,
            "unique_index_unavailable",
        ),
        (
            WorthQueryMergeUniqueValueDenialKind::TargetHeadUnavailable,
            EffectExecutionDenialKind::MergeTargetHeadUnavailable,
            "merge_target_head_unavailable",
        ),
        (
            WorthQueryMergeUniqueValueDenialKind::MergedEntityNotLive,
            EffectExecutionDenialKind::MergedEntityNotLive,
            "merged_entity_not_live",
        ),
        (
            WorthQueryMergeUniqueValueDenialKind::UnreadWriteShape,
            EffectExecutionDenialKind::MergeWriteShapeUnread,
            "merge_write_shape_unread",
        ),
    ] {
        assert_eq!(merge_unique_value_denial_kind(refusal), kind);
        assert_eq!(kind.as_str(), name);
    }
}
