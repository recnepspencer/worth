//! Optional managed-view work is admitted against the sealed candidate before World effect.

use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::CommitId,
    identity::VersionId,
    mvcc::{PreparedRelationalChangeSummaryBudget, PreparedRelationalCommitCandidate},
    publication::RecordStructuralChange,
    snapshots::SnapshotHandle,
    transactions::{CommitResult, RecordRef},
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use crate::domain_computation::{
    execution_runtime::product_world::{
        WorthQueryProductPublicationBinding, WorthQueryProductPublicationReceipt,
    },
    primary_graph::{
        application_query::derived_view::{
            changes_from_summary, PreparedManagedViewPublication, ViewPublicationBasis,
        },
        provider::WorthQueryPrimaryGraphProvider,
    },
};

const MAXIMUM_CHANGED_RECORDS: usize = 100_000;
const MAXIMUM_ASPECT_SCOPES: usize = 100_000;
const MAXIMUM_VIEW_CHANGES: usize = 500_000;
const MAXIMUM_VIEW_WORK_UNITS: usize = 1_000_000;

pub(super) struct PreparedViewPublication {
    plan: PreparedManagedViewPublication,
    expected_after: Option<(u64, VersionId, CommitId)>,
}

pub(super) fn prepare(
    provider: &WorthQueryPrimaryGraphProvider,
    product: &WorthQueryProductPublicationBinding,
    before: &SnapshotHandle,
    candidate: &PreparedRelationalCommitCandidate,
) -> Option<PreparedViewPublication> {
    let trace = std::env::var_os("WORTH_SCENE_TRACE").is_some();
    let registry = provider.graph.managed_derived_views();
    if !registry.has_live_views() {
        return None;
    }
    let observation = product.observation();
    let relational = observation.basis().relational_basis();
    let descriptor = relational.descriptor();
    let basis = ViewPublicationBasis {
        before: observation.selected_commit(),
        relational_branch: relational.identity().branch_id(),
        product_branch: observation.branch_identity(),
        incarnation: observation.lifecycle_incarnation(),
    };
    let summary = provider.graph.with_runtime(|runtime| {
        runtime.preparation_port().summarize_prepared_candidate(
            candidate,
            PreparedRelationalChangeSummaryBudget {
                max_records: MAXIMUM_CHANGED_RECORDS,
                max_aspect_scopes: MAXIMUM_ASPECT_SCOPES,
            },
        )
    });
    if trace {
        match &summary {
            Ok(summary) => eprintln!(
                "managed view sealed summary: records={} scopes={} updated_empty_scopes={}",
                summary.records.len(),
                summary
                    .records
                    .iter()
                    .map(|record| record.aspect_scopes.len())
                    .sum::<usize>(),
                summary
                    .records
                    .iter()
                    .filter(|record| matches!(record.target, RecordRef::Entity(_))
                        && record.structural_change == RecordStructuralChange::Updated
                        && record.aspect_scopes.is_empty())
                    .count(),
            ),
            Err(_) => eprintln!("managed view sealed summary: denied"),
        }
    }
    let admissible = summary.ok().filter(|summary| {
        summary.runtime_instance_id == before.runtime_instance_id()
            && summary.runtime_instance_id == descriptor.runtime_instance_id()
            && &summary.branch_id == before.branch_id()
            && &summary.branch_id == descriptor.branch_id()
            && summary.before_version == before.version_id()
            && summary.before_root_identity == descriptor.root_identity()
    });
    let changes = admissible
        .as_ref()
        .and_then(|summary| changes_from_summary(summary, MAXIMUM_VIEW_CHANGES));
    if trace {
        eprintln!(
            "managed view summary admissible={} changes={:?}",
            admissible.is_some(),
            changes.as_ref().map(Vec::len)
        );
    }
    let has_changes = changes.is_some();
    let plan = match changes {
        Some(changes) => registry.prepare_publication(basis, &changes, MAXIMUM_VIEW_WORK_UNITS),
        None => registry.prepare_publication(basis, &[], 0),
    };
    let expected_after = admissible.filter(|_| has_changes).map(|summary| {
        (
            summary.after_root_identity,
            summary.after_version,
            summary.after_commit_id,
        )
    });
    Some(PreparedViewPublication {
        plan,
        expected_after,
    })
}

impl PreparedViewPublication {
    pub(super) fn apply(
        self,
        committed: &CommitResult,
        next_basis: &AdmittedRelationalBranchBasis,
        product: &WorthQueryProductPublicationReceipt,
    ) {
        let after: &CompositeCommitIdentity = product.publication().commit().identity();
        let exact = self.expected_after.is_some_and(|(root, version, commit)| {
            next_basis.descriptor().root_identity() == root
                && committed.snapshot.version_id() == version
                && committed.commit.commit_id == commit
        });
        if std::env::var_os("WORTH_SCENE_TRACE").is_some() {
            eprintln!("managed view publication exact={exact}");
        }
        if exact {
            self.plan.apply(after);
        } else {
            self.plan.apply_cold(after);
        }
    }
}
