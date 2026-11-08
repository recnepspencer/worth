//! Admit current program support and original attribution on every real branch.
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectValue, CanonicalDigestId, DurableCompositeCommitIdentity, InternedString,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::*;

#[derive(Debug)]
pub enum QueryHomeProgramActivationDenial {
    SupportUnavailable,
    Execution(worth_execution::WorkCeilingDenial),
    Inventory(RuntimeWorldServiceDenial<ProductBranchInventoryDenial>),
    Source(RuntimeWorldServiceDenial<WorldReadmissionDenial>),
    History(RuntimeWorldServiceDenial<ProductExactHistoryDenial>),
    Continuation(ProductExactHistoryDenial),
    Native(WorthQueryPrimaryGraphInstallationDenial),
    UnrosteredCurrentActivation {
        branch: ProductBranchIdentity,
    },
    RetiredCurrentSupport {
        branch: ProductBranchIdentity,
    },
    MissingOriginal {
        commit: CompositeCommitIdentity,
    },
    HistoryProgramMismatch {
        branch: ProductBranchIdentity,
        commit: CompositeCommitIdentity,
    },
    HistoryPredecessorMismatch {
        branch: ProductBranchIdentity,
    },
}
use QueryHomeProgramActivationDenial as Denial;

pub(in crate::domain_computation::primary_graph) fn verify<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let support = application
        .installed_program_support()
        .ok_or(Denial::SupportUnavailable)?;
    let owner = &application.runtime.product_runtime.owner;
    let observation = owner.observation_port();
    let inventory = observation
        .observe_installed_product_branches(execution)
        .map_err(Denial::Inventory)?;
    let originals = owner
        .inspection_port()
        .observe_program_activation_history(execution)
        .map_err(Denial::Source)?;
    for branch in inventory.branches() {
        execution.consult().map_err(Denial::Execution)?;
        let current = activation(application, execution, branch.basis().relational_basis())?;
        let Some(program) = support.rostered_for_rendering(&current) else {
            return Err(Denial::UnrosteredCurrentActivation {
                branch: branch.branch_identity().clone(),
            });
        };
        if support.present(program.revision()).is_none() {
            return Err(Denial::RetiredCurrentSupport {
                branch: branch.branch_identity().clone(),
            });
        }
        verify_history(application, execution, &observation, &originals, branch)?;
    }
    Ok(())
}

fn verify_history<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    observation: &RuntimeWorldObservationPort,
    originals: &WorldProgramActivationHistory,
    branch: &ProductBranchObservation,
) -> Result<(), Denial> {
    let mut cursor = observation
        .open_exact_product_history(*execution, branch)
        .map_err(Denial::History)?;
    let mut predecessor: Option<(DurableCompositeCommitIdentity, [u8; 32])> = None;
    loop {
        execution.consult().map_err(Denial::Execution)?;
        let prepared = observation
            .prepare_exact_product_history_page(*execution, &cursor)
            .map_err(Denial::History)?;
        let (page, next) = cursor.accept_page(prepared).map_err(Denial::Continuation)?;
        if let Some(commit) = page.commit() {
            let value = activation(application, execution, commit.basis().relational_basis())?;
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
            return Ok(());
        }
        cursor = next;
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
    if !matches!(value, AspectValue::String(InternedString::Raw(text)) if *text == CanonicalDigestId::new(revision).render_hex())
    {
        return Err(Denial::HistoryProgramMismatch {
            branch: branch.branch_identity().clone(),
            commit: commit.clone(),
        });
    }
    Ok(())
}
fn activation<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<AspectValue, Denial> {
    let support = application
        .installed_program_support()
        .ok_or(Denial::SupportUnavailable)?;
    let published = support
        .activation()
        .published()
        .ok_or(Denial::SupportUnavailable)?;
    let graph = &application.runtime.primary_provider.graph;
    graph.with_runtime(execution, |native| {
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
        let (identity, _) = super::read_activation(
            execution,
            &view,
            graph.layout.program_activation(),
            usize::MAX,
            "branch activation could not be read",
            |rendering| {
                value = Some(rendering.clone());
                true
            },
        )
        .map_err(Denial::Native)?;
        if identity != published {
            return Err(Denial::SupportUnavailable);
        }
        Ok(value.expect("successful exact activation read visits its value"))
    })
}
