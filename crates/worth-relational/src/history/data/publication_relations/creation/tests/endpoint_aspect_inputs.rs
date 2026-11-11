//! Descriptive birth twins with optional endpoint-aspect operations removed.
use super::{AspectBinding, Op, PublishedAuthoritativePatch};

pub(super) fn without_endpoint_aspects(
    patch: &PublishedAuthoritativePatch,
) -> PublishedAuthoritativePatch {
    PublishedAuthoritativePatch::new(
        patch
            .full_grammar_operations()
            .iter()
            .filter(|operation| {
                !matches!(
                    operation,
                    Op::WholeAspectSet {
                        binding: AspectBinding::RelationSourceEndpoint
                            | AspectBinding::RelationTargetEndpoint,
                        ..
                    }
                )
            })
            .cloned()
            .collect(),
    )
}
