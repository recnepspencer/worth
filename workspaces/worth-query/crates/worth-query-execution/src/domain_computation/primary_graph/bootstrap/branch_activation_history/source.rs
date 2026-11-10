//! Concrete Native reads and exact World original attribution on one selected history.
use super::{
    Denial, WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectValue, CanonicalDigestId, DurableCompositeCommitIdentity};
use worth_relational::facade::{identity::EntityId, runtime::RelationalRuntime};
use worth_runtime_world::facade::*;

pub(in crate::domain_computation::primary_graph) struct BranchActivationSource<'a> {
    native: &'a RelationalRuntime,
    layout: &'a WorthQueryPrimaryGraphLayout,
    published: EntityId,
}
impl<'a> BranchActivationSource<'a> {
    pub(in crate::domain_computation::primary_graph) const fn new(
        native: &'a RelationalRuntime,
        layout: &'a WorthQueryPrimaryGraphLayout,
        published: EntityId,
    ) -> Self {
        Self {
            native,
            layout,
            published,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn at_branch(
        native: &'a RelationalRuntime,
        layout: &'a WorthQueryPrimaryGraphLayout,
        execution: &ExecutionRequest<'_, '_>,
        branch: &ProductBranchObservation,
    ) -> Result<(Self, AspectValue), Denial> {
        let (published, current) =
            read_record(execution, native, layout, branch.basis().relational_basis())?;
        Ok((Self::new(native, layout, published), current))
    }
    pub(in crate::domain_computation::primary_graph) fn read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    ) -> Result<AspectValue, Denial> {
        let (identity, value) = read_record(execution, self.native, self.layout, basis)?;
        if identity != self.published {
            return Err(Denial::SupportUnavailable);
        }
        Ok(value)
    }

    pub(in crate::domain_computation::primary_graph) fn history(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &RuntimeWorldObservationPort,
        originals: &WorldProgramActivationHistory,
        branch: &ProductBranchObservation,
    ) -> Result<std::collections::BTreeSet<CanonicalDigestId>, Denial> {
        let mut cursor = observation
            .open_exact_product_history(*execution, branch)
            .map_err(Denial::History)?;
        let mut revisions = std::collections::BTreeSet::new();
        let mut predecessor: Option<(DurableCompositeCommitIdentity, [u8; 32])> = None;
        loop {
            execution.consult().map_err(Denial::Execution)?;
            let prepared = observation
                .prepare_exact_product_history_page(*execution, &cursor)
                .map_err(Denial::History)?;
            let (page, next) = cursor.accept_page(prepared).map_err(Denial::Continuation)?;
            if let Some(commit) = page.commit() {
                let value = self.read(execution, commit.basis().relational_basis())?;
                revisions.insert(recorded_revision(&value)?);
                if let Some((expected, revision)) = predecessor.take() {
                    if commit.identity().owner_identity().durable() != Some(expected.owner())
                        || commit.identity().ordinal() != expected.ordinal()
                    {
                        return Err(Denial::HistoryPredecessorMismatch {
                            branch: branch.branch_identity().clone(),
                        });
                    }
                    compare(&value, revision, branch, commit.identity())?;
                }
                let entry = originals
                    .entry(commit.identity())
                    .map_err(|cause| Denial::Source(RuntimeWorldServiceDenial::Denied(cause)))?
                    .ok_or_else(|| Denial::MissingOriginal {
                        commit: commit.identity().clone(),
                    })?;
                let revision = entry
                    .expected_activation(execution)
                    .map_err(|cause| Denial::Source(RuntimeWorldServiceDenial::Denied(cause)))?;
                revisions.insert(CanonicalDigestId::new(revision));
                match entry.predecessor() {
                    Some(parent) => predecessor = Some((parent, revision)),
                    None => compare(&value, revision, branch, commit.identity())?,
                }
            }
            if page.is_complete() {
                if predecessor.is_some() {
                    return Err(Denial::HistoryPredecessorMismatch {
                        branch: branch.branch_identity().clone(),
                    });
                }
                return Ok(revisions);
            }
            cursor = next;
        }
    }
}

fn compare(
    value: &AspectValue,
    revision: [u8; 32],
    branch: &ProductBranchObservation,
    commit: &CompositeCommitIdentity,
) -> Result<(), Denial> {
    // Compare DATA with the existing one-way revision rendering. Never mint a
    // program revision or authority from an original's recorded digest.
    let expected = CanonicalDigestId::new(revision).render_hex();
    if value.raw_text() != Some(expected.as_str()) {
        return Err(Denial::HistoryProgramMismatch {
            branch: branch.branch_identity().clone(),
            commit: commit.clone(),
        });
    }
    Ok(())
}
fn recorded_revision(value: &AspectValue) -> Result<CanonicalDigestId, Denial> {
    let text = value.raw_text().ok_or(Denial::SupportUnavailable)?;
    CanonicalDigestId::parse_hex(text)
        .filter(|digest| digest.render_hex() == text)
        .ok_or(Denial::SupportUnavailable)
}

fn read_record(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<(EntityId, AspectValue), Denial> {
    let view = native
        .read_truth()
        .project_observation(&basis.observation())
        .map_err(|cause| {
            Denial::Native(WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                format!("branch activation observation refused: {cause:?}"),
            ))
        })?;
    let mut value = None;
    let (identity, _) = super::super::read_activation(
        execution,
        &view,
        layout.program_activation(),
        usize::MAX,
        "branch activation could not be read",
        |rendering| {
            value = Some(rendering.clone());
            true
        },
    )
    .map_err(Denial::Native)?;
    Ok((
        identity,
        value.expect("successful exact activation read visits its value"),
    ))
}
