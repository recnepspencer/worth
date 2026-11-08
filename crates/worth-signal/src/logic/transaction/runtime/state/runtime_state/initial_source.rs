//! Prepare the actual original program's first authored definition and branch.
use super::SignalRuntime;
use crate::durability::initial_source::{InitialSourceCustody, InitialSourcePhase};
use crate::durability::readmission::{
    SignalAdvanceLocationReservation, SignalDefinitionLocationReservation,
};
use crate::durability::{
    SignalAuthoredBranchChange, SignalAuthoredInstallationRefused, SignalBranchAdvanceArtifact,
    SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor,
    SignalDurableMedium, SignalDurableTierVocabulary, SignalInitialDefinitionContext,
    SignalInitialSourceDenial as D, SignalInitialSourcePreparationRefusal,
    SignalInitialSourcePublication, SignalPublicationTaskPosture,
    SignalRecordedIdentityContribution,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalConditionalDefinitionOccurrence,
    DurableSignalGeneration, DurableSignalNodeIdentity, SignalDurableJournal,
};

impl<Dm, I, E, Ctx, T> SignalRuntime<Dm, I, E, Ctx, T>
where
    Dm: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Consume the real initial program. The section callback captures authored
    /// declarations only; no publication or owner effect has started on refusal.
    pub fn prepare_initial_authored_source<F>(
        self: Box<Self>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
        vocabulary: SignalDurableTierVocabulary<T>,
        execution: &ExecutionRequest<'_, '_>,
        section: F,
    ) -> Result<
        SignalInitialSourcePublication<Dm, I, E, Ctx, T>,
        SignalInitialSourcePreparationRefusal<Self>,
    >
    where
        F: FnOnce(
            SignalInitialDefinitionContext<'_>,
        )
            -> Result<SignalBridgeAuthoredSection, SignalAuthoredInstallationRefused>,
    {
        let prepared = self.prepare_initial_frames(
            medium.as_ref(),
            contribution,
            &vocabulary,
            execution,
            section,
        );
        let PreparedInitialFrames {
            definition,
            advance,
            contribution,
            admission,
            definition_custody,
            advance_custody,
            memory,
        } = match prepared {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(SignalInitialSourcePreparationRefusal {
                    program: self,
                    cause,
                })
            }
        };
        Ok(SignalInitialSourcePublication {
            custody: Box::new(InitialSourceCustody {
                program: self,
                medium,
                contribution,
                vocabulary,
                admission,
                definition,
                advance,
                definition_custody: Some(definition_custody),
                _advance_custody: advance_custody,
                _memory: memory,
            }),
            phase: InitialSourcePhase::Definition,
        })
    }

    fn prepare_initial_frames<F>(
        &self,
        medium: &dyn SignalDurableMedium,
        mut contribution: SignalRecordedIdentityContribution,
        vocabulary: &SignalDurableTierVocabulary<T>,
        execution: &ExecutionRequest<'_, '_>,
        section: F,
    ) -> Result<PreparedInitialFrames, D>
    where
        F: FnOnce(
            SignalInitialDefinitionContext<'_>,
        )
            -> Result<SignalBridgeAuthoredSection, SignalAuthoredInstallationRefused>,
    {
        if self.owner_services.is_sealed() {
            return Err(D::Readmission(
                crate::durability::SignalOwnerReadmissionDenial::ProgramAlreadyServing,
            ));
        }
        self.owner_service_issuance_capability().map_err(|cause| {
            D::Readmission(crate::durability::SignalOwnerReadmissionDenial::OwnerServices(cause))
        })?;
        let handle = self.current_branch();
        if handle.id.0 != 0
            || handle.parent_branch_id.is_some()
            || handle.head_snapshot_id.is_some()
            || !self.branches.initial_authored_source_is_pristine()
        {
            return Err(D::ProgramNotInitial);
        }
        if contribution.owner() != medium.binding().owner() {
            return Err(D::Readmission(
                crate::durability::SignalOwnerReadmissionDenial::ForeignOwner,
            ));
        }
        let admission = medium
            .owner_admission_registry()
            .admit(medium.binding(), self.branches.owner_runtime_instance_id())
            .map_err(D::Medium)?;
        match medium.publication_task_posture() {
            SignalPublicationTaskPosture::Terminal => {}
            posture @ (SignalPublicationTaskPosture::Busy
            | SignalPublicationTaskPosture::Unsettled) => {
                return Err(D::Readmission(
                    crate::durability::SignalOwnerReadmissionDenial::PublicationNotQuiescent(
                        posture,
                    ),
                ))
            }
        }
        let source = medium
            .capture_recovery_source(execution)
            .map_err(D::Medium)?;
        if source.owner() != medium.binding().owner() {
            return Err(D::Readmission(
                crate::durability::SignalOwnerReadmissionDenial::ForeignOwner,
            ));
        }
        let definitions = source
            .frontier(SignalDurableJournal::Definitions, execution)
            .map_err(D::Medium)?;
        let advances = source
            .frontier(SignalDurableJournal::Advances, execution)
            .map_err(D::Medium)?;
        if definitions != 0 || advances != 0 {
            return Err(D::SourceNotEmpty {
                definitions,
                advances,
            });
        }
        contribution
            .merge(&source.identity_contribution(), execution)
            .map_err(D::Readmission)?;
        let branch = DurableSignalBranchIdentity::from_recorded(medium.binding().owner(), 0);
        contribution
            .observe_graph(branch, &self.graph, execution)
            .map_err(D::Readmission)?;
        let identity = contribution.issue_definition().map_err(D::Readmission)?;
        let cell = contribution
            .issue_fork_cell(branch)
            .map_err(D::Readmission)?;
        let (advance_identity, admission_identity) = contribution
            .issue_advance_and_admission()
            .map_err(D::Readmission)?;
        let count = self.graph.authored_conditional_nodes().count();
        let bytes = count
            .checked_mul(std::mem::size_of::<
                DurableSignalConditionalDefinitionOccurrence,
            >())
            .ok_or(D::Readmission(
                crate::durability::SignalOwnerReadmissionDenial::MetadataCapacity,
            ))?;
        let _scratch = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| {
                D::Readmission(crate::durability::SignalOwnerReadmissionDenial::Request(
                    cause,
                ))
            })?;
        let mut occurrences = Vec::with_capacity(count);
        for node in self.graph.authored_conditional_nodes() {
            let node =
                DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation());
            occurrences.push(
                DurableSignalConditionalDefinitionOccurrence::from_recorded(
                    node,
                    contribution.issue_conditional().map_err(D::Readmission)?,
                )
                .expect("owner issues the actual node's occurrence"),
            );
        }
        let section = section(SignalInitialDefinitionContext {
            graph: &self.graph,
            identity,
            branch,
            occurrences: &occurrences,
        })
        .map_err(|_| D::BridgeSectionRefused)?;
        let image = self
            .graph
            .capture_authored_image(execution, branch)
            .map_err(D::Image)?;
        let runtime = self
            .config
            .capture_authored_image(&self.graph, vocabulary, branch, execution)
            .map_err(D::Image)?;
        let definition = SignalDefinitionTransitionArtifact::capture(
            execution,
            identity,
            branch,
            None,
            &image,
            &runtime,
            &occurrences,
            &section,
        )
        .map_err(D::Frame)?;
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(cell, 0),
            None,
            admission_identity,
            definition.reference(),
        )
        .map_err(D::Frame)?;
        let advance = SignalBranchAdvanceArtifact::capture(
            execution,
            advance_identity,
            descriptor,
            None,
            SignalAuthoredBranchChange::Created {
                name: handle.name,
                parent: None,
            },
        )
        .map_err(D::Frame)?;
        contribution.check_issuable().map_err(D::Readmission)?;
        let definition_custody =
            SignalDefinitionLocationReservation::reserve(execution).map_err(D::Readmission)?;
        let advance_custody =
            SignalAdvanceLocationReservation::reserve(execution).map_err(D::Readmission)?;
        let memory = execution
            .reserve_retained_memory(
                (std::mem::size_of::<InitialSourceCustody<Dm, I, E, Ctx, T>>()
                    + std::mem::size_of::<SignalInitialSourcePublication<Dm, I, E, Ctx, T>>())
                    as u64,
            )
            .map_err(|cause| {
                D::Readmission(crate::durability::SignalOwnerReadmissionDenial::Request(
                    cause,
                ))
            })?;
        Ok(PreparedInitialFrames {
            definition,
            advance,
            contribution,
            admission,
            definition_custody,
            advance_custody,
            memory,
        })
    }
}
struct PreparedInitialFrames {
    definition: SignalDefinitionTransitionArtifact,
    advance: SignalBranchAdvanceArtifact,
    contribution: SignalRecordedIdentityContribution,
    admission: crate::durability::SignalDurableOwnerAdmission,
    definition_custody: SignalDefinitionLocationReservation,
    advance_custody: SignalAdvanceLocationReservation,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}
