//! One native prepared attempt per owned node, including across branch cells.

use std::sync::Arc;

use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;

use super::RequiredWorkMembership;
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::native_hints::NativeBranch;
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::native_hints::NativeHint;




impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) fn prepared_native_branch(
        branch: BranchId,
    ) -> Arc<NativeBranch> {
        Arc::new(NativeBranch { branch })
    }

    /// Construction happens only after native preflight pays the Box and
    /// retained ticket. Installation only links this already-owned node.
    pub(in crate::domain_computation::primary_graph) fn prepared_native_hint(
        observer: CompanionPublicationCompletionObserver,
        branch: Arc<NativeBranch>,
    ) -> Box<NativeHint> {
        Box::new(NativeHint {
            observer,
            branch,
            seen_settled_at: None,
            next: None,
        })
    }
}


mod court_doors_for_test;
