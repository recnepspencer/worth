pub use crate::correspondence::RuntimeWorldCorrespondenceInspectionCounters;
use crate::correspondence::RuntimeWorldCorrespondenceInspectionLedger;
use crate::correspondence::{
    admit_baseline, admit_installed_basis, admit_installed_basis_in, compare_current_basis,
    compare_current_basis_in, AdmittedRuntimeWorldCorrespondenceBasis,
    RuntimeWorldCorrespondenceAdmissionDenial,
};
pub use crate::correspondence::{RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal};
use std::sync::Arc;

use crate::facade::{BridgeInstalledSemanticCorrespondence, RuntimeBridge};

/// Narrow Bridge admission port for the Runtime World composition owner.
///
/// It admits an already-installed Bridge correspondence and exposes no
/// mapping, Signal, or Relational construction surface.
#[derive(Clone)]
pub struct RuntimeWorldCorrespondencePort {
    pub(crate) runtime: RuntimeBridge,
    pub(crate) branches: RuntimeWorldCorrespondenceBranches,
    pub(crate) inspection: Arc<RuntimeWorldCorrespondenceInspectionLedger>,
}

#[derive(Clone)]
pub(crate) enum RuntimeWorldCorrespondenceBranches {
    Memory,
    Authored(
        std::sync::Weak<
            crate::conditional_execution::authored_section::branch_registry::BranchRegistry,
        >,
    ),
}
impl RuntimeWorldCorrespondenceBranches {
    fn with_registry<R>(
        &self,
        graph: u64,
        signal_branch: Option<&worth_signal::facade::branch::SignalBranchIdentity>,
        memory: &crate::correspondence::AdmittedSemanticDependencyRegistry,
        inspect: impl FnOnce(&crate::correspondence::AdmittedSemanticDependencyRegistry) -> R,
    ) -> Result<R, RuntimeWorldCorrespondenceAdmissionDenial> {
        match self {
            Self::Memory => Ok(inspect(memory)),
            Self::Authored(owner) => owner
                .upgrade()
                .ok_or(RuntimeWorldCorrespondenceAdmissionDenial::AuthoredOwnerUnavailable)?
                .with_branch_registry(
                    graph,
                    signal_branch.ok_or(
                        RuntimeWorldCorrespondenceAdmissionDenial::SignalBranchBasisRequired,
                    )?,
                    inspect,
                ),
        }
    }
}
impl std::fmt::Debug for RuntimeWorldCorrespondencePort {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeWorldCorrespondencePort")
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl RuntimeWorldCorrespondencePort {
    pub fn export_recorded_runtime_world_baseline_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal> {
        if basis.dependency().is_some() {
            return Err(RuntimeWorldBaselineRefusal::BaselineCorrespondenceRequired);
        }
        self.compare_current_exact(basis)
            .map_err(|_| RuntimeWorldBaselineRefusal::CurrentBasisRequired)?;
        crate::correspondence::recorded_baseline_in(execution, &self.runtime)
    }

    pub fn export_recorded_runtime_world_baseline(
        &self,
        basis: &AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal> {
        if basis.dependency().is_some() {
            return Err(RuntimeWorldBaselineRefusal::BaselineCorrespondenceRequired);
        }
        self.compare_current_exact(basis)
            .map_err(|_| RuntimeWorldBaselineRefusal::CurrentBasisRequired)?;
        crate::correspondence::recorded_baseline(&self.runtime)
    }

    pub fn readmit_recorded_runtime_world_baseline(
        &self,
        recorded: &RecordedRuntimeWorldBaseline,
        signal: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldBaselineRefusal> {
        let admitted = crate::correspondence::readmit_baseline(&self.runtime, recorded, signal)?;
        self.compare_current_exact(&admitted)
            .map_err(|_| RuntimeWorldBaselineRefusal::CurrentBasisRequired)?;
        Ok(admitted)
    }

    pub fn admit_installed_basis(
        &self,
        installed: &BridgeInstalledSemanticCorrespondence,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondenceAdmissionDenial>
    {
        match &self.branches {
            RuntimeWorldCorrespondenceBranches::Memory => {
                admit_installed_basis(&self.runtime, installed, &self.inspection)
            }
            RuntimeWorldCorrespondenceBranches::Authored(_) => self.branches.with_registry(
                installed.basis().signal_graph_instance_id,
                None,
                &self.runtime.semantic_dependency_registry,
                |registry| {
                    admit_installed_basis_in(&self.runtime, registry, installed, &self.inspection)
                },
            )?,
        }
    }

    pub fn compare_current_exact(
        &self,
        admitted: &AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<(), RuntimeWorldCorrespondenceAdmissionDenial> {
        match &self.branches {
            RuntimeWorldCorrespondenceBranches::Memory => {
                compare_current_basis(&self.runtime, admitted, &self.inspection)
            }
            RuntimeWorldCorrespondenceBranches::Authored(_) => self.branches.with_registry(
                admitted.signal_graph_instance_id(),
                admitted.signal_branch(),
                &self.runtime.semantic_dependency_registry,
                |registry| {
                    compare_current_basis_in(&self.runtime, registry, admitted, &self.inspection)
                },
            )?,
        }
    }

    pub fn admit_runtime_baseline(
        &self,
        signal_graph_instance_id: u64,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondenceAdmissionDenial>
    {
        self.branches.with_registry(
            signal_graph_instance_id,
            None,
            &self.runtime.semantic_dependency_registry,
            |_| admit_baseline(&self.runtime, signal_graph_instance_id),
        )
    }

    pub fn admit_signal_baseline(
        &self,
        signal: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondenceAdmissionDenial>
    {
        let graph = signal
            .observation()
            .target()
            .as_basis()
            .and_then(|basis| basis.graph_instance_id().parse::<u64>().ok())
            .ok_or(RuntimeWorldCorrespondenceAdmissionDenial::SignalBranchBasisRequired)?;
        self.branches.with_registry(
            graph,
            Some(signal.observation().branch_id()),
            &self.runtime.semantic_dependency_registry,
            |_| admit_baseline(&self.runtime, graph).bind_signal_branch(signal),
        )
    }
    pub fn admit_installed_basis_at_signal_basis(
        &self,
        installed: &BridgeInstalledSemanticCorrespondence,
        signal: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondenceAdmissionDenial>
    {
        let baseline = self.admit_signal_baseline(signal)?;
        if baseline.signal_graph_instance_id() != installed.basis().signal_graph_instance_id {
            return Err(
                RuntimeWorldCorrespondenceAdmissionDenial::InstalledCorrespondenceNotCurrent,
            );
        }
        self.branches.with_registry(
            baseline.signal_graph_instance_id(),
            baseline.signal_branch(),
            &self.runtime.semantic_dependency_registry,
            |registry| {
                admit_installed_basis_in(&self.runtime, registry, installed, &self.inspection)
                    .map(|basis| basis.bind_signal_branch(signal))
            },
        )?
    }
    pub fn inspection_counters(&self) -> RuntimeWorldCorrespondenceInspectionCounters {
        self.inspection.snapshot()
    }
}

impl RuntimeBridge {
    /// Borrow the Bridge's exact installed-correspondence admission seam.
    pub fn runtime_world_correspondence_port(&self) -> RuntimeWorldCorrespondencePort {
        RuntimeWorldCorrespondencePort {
            runtime: self.clone(),
            branches: RuntimeWorldCorrespondenceBranches::Memory,
            inspection: Arc::new(RuntimeWorldCorrespondenceInspectionLedger::default()),
        }
    }
}
