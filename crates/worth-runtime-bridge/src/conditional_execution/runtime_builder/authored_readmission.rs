//! Consume the program owner, then seal fresh Bridge projections after Signal admission.
use super::BridgeConditionalRuntimeBuilder;
use crate::conditional_execution::authored_section::branch_projection::{
    BranchPosture, BranchProjection, ReadmittedBranch,
};
use crate::conditional_execution::authored_section::readmission::{
    Installation, PreparedBranch, Program,
};
use crate::conditional_execution::{
    BridgeAuthoredInstallationBindings, BridgeAuthoredReadmissionDenial as D,
    BridgeOwnedSignalRuntime, BridgeReadmittedAuthoredAssembly,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::{
    SignalDurableMedium, SignalDurableTierVocabulary, SignalRecordedIdentityContribution,
};

pub(super) enum AuthoredSignalSource {
    Recovered {
        program: Box<worth_signal::facade::SignalRuntime<(), (), (), (), ()>>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
    },
    Initial(worth_signal::facade::durability::SignalPublishedInitialSource<(), (), (), (), ()>),
}

impl BridgeConditionalRuntimeBuilder {
    /// Rebuild all recovered branches before exposing the durable owner assembly.
    pub fn readmit_authored_owner(
        mut self: Box<Self>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
        bindings: &mut dyn BridgeAuthoredInstallationBindings,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<BridgeReadmittedAuthoredAssembly>, D> {
        let source = AuthoredSignalSource::Recovered {
            program: self.take_signal(),
            medium,
            contribution,
        };
        self.readmit_source(source, bindings, execution)
    }
    pub(super) fn readmit_source(
        mut self: Box<Self>,
        source: AuthoredSignalSource,
        bindings: &mut dyn BridgeAuthoredInstallationBindings,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<BridgeReadmittedAuthoredAssembly>, D> {
        if !self.managed_clocks.is_empty() || self.runtime.owned_async_declaration_count() != 0 {
            return Err(D::UnsupportedProgramFamily);
        }
        self.runtime.revoke_conditional_liveness();
        self.runtime.retention.close();
        let mut template = self
            .runtime
            .bridge
            .take()
            .ok_or(D::UnsupportedProgramFamily)?;
        template.diagnostics = template.diagnostics.for_authored_installation();
        template.semantic_dependency_registry = Default::default();
        template.correspondence_allocations = Default::default();
        let _installation_memory = execution
            .reserve_retained_memory(std::mem::size_of::<Installation<'_>>() as u64)
            .map_err(|cause| {
                D::section(
                    crate::conditional_execution::BridgeAuthoredSectionDenial::Request(cause),
                )
            })?;
        let program = Program::new(template, execution)?;
        let mut installation = Box::new(Installation {
            program,
            bindings,
            branches: BTreeMap::new(),
            refusal: None,
        });
        let signal = match source {
            AuthoredSignalSource::Recovered {
                program,
                medium,
                contribution,
            } => {
                let vocabulary = SignalDurableTierVocabulary::declare([((), "bridge")])
                    .map_err(|_| D::ConditionalDefinitionMismatch)?;
                program.readmit_authored_owner(
                    medium,
                    contribution,
                    vocabulary,
                    &mut *installation,
                    execution,
                )
            }
            AuthoredSignalSource::Initial(source) => {
                source.readmit_authored_owner(&mut *installation, execution)
            }
        };
        let signal = match signal {
            Ok(signal) => signal,
            Err(cause) => return Err(installation.refusal.take().unwrap_or(D::signal(cause))),
        };
        let owner_bytes = crate::conditional_execution::retention::sum(&[
            std::mem::size_of::<BridgeReadmittedAuthoredAssembly>() as u64,
            crate::conditional_execution::retention::arc_charge::<
                crate::conditional_execution::authored_section::branch_registry::BranchRegistry,
            >()
            .map_err(|_| {
                D::section(crate::conditional_execution::BridgeAuthoredSectionDenial::SizeLimit)
            })?,
            crate::conditional_execution::retention::btree_charge::<
                worth_foundational::facade::DurableSignalBranchIdentity,
                crate::conditional_execution::authored_section::branch_registry::BranchSlot,
            >(installation.branches.len())
            .map_err(|_| {
                D::section(crate::conditional_execution::BridgeAuthoredSectionDenial::SizeLimit)
            })?,
        ])
        .map_err(|_| {
            D::section(crate::conditional_execution::BridgeAuthoredSectionDenial::SizeLimit)
        })?;
        let memory = execution
            .reserve_retained_memory(owner_bytes)
            .map_err(|cause| {
                D::section(
                    crate::conditional_execution::BridgeAuthoredSectionDenial::Request(cause),
                )
            })?;
        let port = signal.source_port();
        let mut branches = BTreeMap::new();
        for (identity, prepared) in std::mem::take(&mut installation.branches) {
            let descriptor = port.observe(identity).map_err(D::source)?;
            let handle = port.readmit_descriptor(&descriptor).map_err(D::source)?;
            let branch_identity = handle.basis().observation().branch_id().clone();
            branches.insert(identity, crate::conditional_execution::authored_section::branch_registry::BranchSlot::Serving {
                projection: finish(prepared, branch_identity, execution)?, _memory: None,
            });
        }
        installation
            .program
            .bridge
            .diagnostics
            .readmit_writeback_diagnostics(&signal);
        Ok(Box::new(BridgeReadmittedAuthoredAssembly {
            program: Arc::clone(&installation.program),
            signal,
            branches:
                crate::conditional_execution::authored_section::branch_registry::BranchRegistry::new(
                    branches,
                ),
            _memory: memory,
        }))
    }
    pub(super) fn take_signal(
        &mut self,
    ) -> Box<worth_signal::facade::SignalRuntime<(), (), (), (), ()>> {
        Box::new(
            self.runtime
                .signal_runtime
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .expect("unsealed program owns its original Signal runtime"),
        )
    }
}
pub(in crate::conditional_execution) fn finish(
    prepared: Box<PreparedBranch>,
    branch: worth_signal::facade::branch::SignalBranchIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Arc<BranchProjection>, D> {
    use crate::conditional_execution::{lowering_registry, retention};
    prepared
        .backing
        .grow(execution, std::mem::size_of::<ReadmittedBranch>() as u64)?;
    prepared.backing.grow(
        execution,
        retention::arc_charge::<BranchProjection>().map_err(|_| {
            D::section(crate::conditional_execution::BridgeAuthoredSectionDenial::SizeLimit)
        })?,
    )?;
    let ledger = retention::BridgeRetentionLedger::new(
        prepared.program.bridge.policy.conditional_retention(),
    )
    .map_err(|cause| {
        D::conditional(crate::conditional_execution::observation_retention::retention_denial(cause))
    })?;
    let mut registry = lowering_registry::BridgeConditionalLoweringRegistry::default();
    for definition in prepared.definitions {
        let projection =
            crate::conditional_execution::lowering_identity::installed_lowering_identity(
                prepared.program.bridge.signal_runtime_key,
                &branch,
                &definition.signal,
                definition.source.contract.identity(),
                &definition.correspondences,
            );
        let bytes = retention::definition_candidate_retained_bytes(
            &branch,
            &definition.source.contract,
            &definition.source.location,
            &definition.providers,
            projection.len(),
            definition.correspondences.len(),
            definition.plan.as_ref(),
        )
        .map_err(|cause| {
            D::conditional(
                crate::conditional_execution::observation_retention::retention_denial(cause),
            )
        })?;
        prepared.backing.grow(execution, bytes)?;
        // Candidate slots are process-local. Cold installation retains bytes only.
        let charge = Arc::new(ledger.reserve(0, 0, bytes).map_err(|cause| {
            D::conditional(
                crate::conditional_execution::observation_retention::retention_denial(cause),
            )
        })?);
        let key = lowering_registry::BridgeConditionalLoweringKey::new(
            branch.clone(),
            &definition.signal,
        );
        let lowering = BridgeOwnedSignalRuntime::build_installed_lowering(
            &prepared.program.bridge,
            key.clone(),
            Arc::clone(&definition.source.location),
            definition.correspondences,
            definition.plan,
            definition.signal,
            None,
            definition.providers,
            definition.provider_admission,
            Some(charge),
            Some(definition.source),
            Default::default(),
        );
        registry.install(key, lowering);
    }
    Ok(Arc::new(BranchProjection {
        state: std::sync::Mutex::new(Box::new(ReadmittedBranch {
            posture: BranchPosture::Serving,
            _dependencies: prepared.dependencies,
            _allocations: prepared.allocations,
            _program: prepared.program,
            section: prepared.section,
            lowerings: registry,
            retention: ledger,
            _backing: prepared.backing,
        })),
    }))
}
