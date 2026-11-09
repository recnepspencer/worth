//! Admit current program support and original attribution on every real branch.
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_execution::ExecutionRequest;
mod source;
pub(in crate::domain_computation::primary_graph) use source::BranchActivationSource;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::*;

#[derive(Debug)]
pub enum QueryHomeProgramActivationDenial {
    SupportUnavailable,
    ForeignBranchInventory,
    SelectedSourceUnavailable,
    Execution(worth_execution::WorkCeilingDenial),
    Inventory(RuntimeWorldServiceDenial<ProductBranchInventoryDenial>),
    Source(RuntimeWorldServiceDenial<WorldReadmissionDenial>),
    History(RuntimeWorldServiceDenial<ProductExactHistoryDenial>),
    Continuation(ProductExactHistoryDenial),
    Native(WorthQueryPrimaryGraphInstallationDenial),
    Workflow(super::super::workflow::readmission::QueryHomeWorkflowReadmissionDenial),
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
    let published = support
        .activation()
        .published()
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
    let graph = &application.runtime.primary_provider.graph;
    graph.with_runtime(execution, |native| {
        let source = BranchActivationSource::new(native, &graph.layout, published);
        for branch in inventory.branches() {
            execution.consult().map_err(Denial::Execution)?;
            let current = source.read(execution, branch.basis().relational_basis())?;
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
            let history = source.history(execution, &observation, &originals, branch)?;
            super::super::workflow::readmission::readmit(
                execution,
                native,
                graph.layout.workflow(),
                branch,
                worth_foundational::facade::CanonicalDigestId::new(*program.revision().as_bytes()),
                &history,
            )
            .map_err(Denial::Workflow)?;
        }
        Ok(())
    })
}

/// Predecessor planning has genuine Native/World owners but no serving Query runtime.
/// The outer source-lens owner admits its sealed request and incarnation first.
pub(in crate::domain_computation::primary_graph) fn verify_predecessor<Schema>(
    native: &worth_relational::facade::runtime::RelationalRuntime,
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    execution: &ExecutionRequest<'_, '_>,
    root: &ProductBranchObservation,
    roster: &worth_query_installation::facade::WorthQueryProgramSupportRoster<Schema>,
    observation: &RuntimeWorldObservationPort,
    inspection: &RuntimeWorldInspectionPort,
) -> Result<(), Denial> {
    let (source, _) = BranchActivationSource::at_branch(native, layout, execution, root)?;
    let inventory = observation
        .observe_installed_product_branches(execution)
        .map_err(Denial::Inventory)?;
    if inventory
        .branches()
        .iter()
        .any(|branch| branch.owner_identity() != root.owner_identity())
    {
        return Err(Denial::ForeignBranchInventory);
    }
    if !inventory.branches().iter().any(|branch| {
        branch.branch_identity() == root.branch_identity()
            && branch.lifecycle_incarnation() == root.lifecycle_incarnation()
            && branch.reference_generation() == root.reference_generation()
            && branch.selected_commit() == root.selected_commit()
    }) {
        return Err(Denial::SelectedSourceUnavailable);
    }
    let originals = inspection
        .observe_program_activation_history(execution)
        .map_err(Denial::Source)?;
    for branch in inventory.branches() {
        let current = source.read(execution, branch.basis().relational_basis())?;
        let Some(program) = roster.entries().iter().find(|program| {
            current
                == worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(program.revision().to_string()),
                )
        }) else {
            return Err(Denial::UnrosteredCurrentActivation {
                branch: branch.branch_identity().clone(),
            });
        };
        let history = source.history(execution, observation, &originals, branch)?;
        super::super::workflow::readmission::readmit(
            execution,
            native,
            layout.workflow(),
            branch,
            worth_foundational::facade::CanonicalDigestId::new(*program.revision().as_bytes()),
            &history,
        )
        .map_err(Denial::Workflow)?;
    }
    Ok(())
}
