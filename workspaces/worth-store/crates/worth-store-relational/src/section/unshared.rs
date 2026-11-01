//! Concrete Home construction phases precede the Native owner's first Arc.
use super::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveredRelationalRuntimeAuthority, RecoveryOutcome},
    runtime::{
        RelationalInitialSchemaInstallationReceipt, RelationalRuntime,
        RelationalRuntimeConstructionDenial, RelationalRuntimeDefinition,
    },
};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, AnchoredRecordMembership, AnchoredRecordReadMembership,
    ServingPhysicalRuntime,
};

pub(super) struct AttachedRelationalRuntime {
    pub(super) runtime: RelationalRuntime,
    pub(super) cold_source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    pub(super) checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
}
pub struct UnrecoveredRelationalSection {
    attached: AttachedRelationalRuntime,
}
pub struct UnsharedReadmittedRelationalSection {
    attached: AttachedRelationalRuntime,
    recovered: RecoveredRelationalRuntimeAuthority,
    outcome: RecoveryOutcome,
}

impl AttachedRelationalRuntime {
    pub(super) fn attach(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        cold_source: Arc<crate::cold_source::StoreColdSource>,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let medium = crate::journal_medium::StoreJournalMedium::new(
            serving.clone(),
            placement,
            cold_source.clone(),
            checkpoint_selection.clone(),
        );
        let runtime = definition.build_durable(Box::new(medium))?;
        Ok(Self {
            runtime,
            cold_source,
            serving,
            checkpoint_selection,
        })
    }
    pub(super) fn share(
        self,
        recovered: Option<RecoveredRelationalRuntimeAuthority>,
        recovery_outcome: Option<RecoveryOutcome>,
    ) -> RelationalStoreSection {
        RelationalStoreSection {
            runtime: Arc::new(self.runtime),
            cold_source: self.cold_source,
            serving: self.serving,
            checkpoint_selection: self.checkpoint_selection,
            recovered,
            recovery_outcome,
        }
    }
}

impl UnrecoveredRelationalSection {
    pub fn prepare_close(
        self,
    ) -> Result<ClosingRelationalSection, (Self, RelationalSectionCloseDenial)> {
        match super::close::prepare_attached_close(self.attached, None, None) {
            Ok(closing) => Ok(closing),
            Err((attached, _, _, cause)) => Err((Self { attached }, cause)),
        }
    }
    pub fn attach_unrecovered_home(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let source = crate::cold_source::StoreColdSource::new(serving.clone());
        Self::attach_with_source(definition, serving, placement, source, checkpoint_selection)
    }
    pub(super) fn attach_with_source(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        source: Arc<crate::cold_source::StoreColdSource>,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        AttachedRelationalRuntime::attach(
            definition,
            serving,
            placement,
            source,
            checkpoint_selection,
        )
        .map(|attached| Self { attached })
    }
    pub fn home_schema_installation_runtime(&mut self) -> &mut RelationalRuntime {
        &mut self.attached.runtime
    }

    pub fn readmit(
        self,
        selected: Option<(&[u8], &AnchoredRecordMembership)>,
        expected: &RelationalInitialSchemaInstallationReceipt,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<UnsharedReadmittedRelationalSection, (Self, RelationalSectionReadmissionDenial)>
    {
        if expected.runtime_instance_id() != self.attached.runtime.runtime_instance_id() {
            return Err((
                self,
                RelationalSectionReadmissionDenial::InitialSchemaRuntimeMismatch,
            ));
        }
        self.readmit_without_schema(selected, execution)
    }
    pub fn readmit_without_schema(
        mut self,
        selected: Option<(&[u8], &AnchoredRecordMembership)>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<UnsharedReadmittedRelationalSection, (Self, RelationalSectionReadmissionDenial)>
    {
        let checkpoint = match super::readmission::prepare_ordinary_source(
            &self.attached.cold_source,
            selected,
            execution,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(cause) => return Err((self, cause)),
        };
        match super::readmission::recover_runtime(
            &mut self.attached.runtime,
            &self.attached.cold_source,
            &self.attached.serving,
            checkpoint,
            execution,
        ) {
            Ok(admitted) => Ok(UnsharedReadmittedRelationalSection {
                attached: self.attached,
                recovered: admitted.authority,
                outcome: admitted.outcome,
            }),
            Err(cause) => Err((self, cause)),
        }
    }

    /// Admit the closed Home's actual selected read before the shared Native
    /// recovery core. A refusal preserves this SAME unshared Native owner.
    pub fn readmit_home_from_read(
        mut self,
        selected: Option<(
            worth_execution::ExecutionImmutableBytes,
            std::ops::Range<usize>,
            &AnchoredRecordReadMembership,
        )>,
        expected: &RelationalInitialSchemaInstallationReceipt,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (UnsharedReadmittedRelationalSection, Option<[u64; 2]>),
        (Self, RelationalSectionReadmissionDenial),
    > {
        if expected.runtime_instance_id() != self.attached.runtime.runtime_instance_id() {
            return Err((
                self,
                RelationalSectionReadmissionDenial::InitialSchemaRuntimeMismatch,
            ));
        }
        let checkpoint = match super::readmission::prepare_home_source(
            &self.attached.cold_source,
            selected,
            execution,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(cause) => return Err((self, cause)),
        };
        match super::readmission::recover_runtime(
            &mut self.attached.runtime,
            &self.attached.cold_source,
            &self.attached.serving,
            checkpoint,
            execution,
        ) {
            Ok(admitted) => Ok((
                UnsharedReadmittedRelationalSection {
                    attached: self.attached,
                    recovered: admitted.authority,
                    outcome: admitted.outcome,
                },
                admitted.selected_coverage,
            )),
            Err(cause) => Err((self, cause)),
        }
    }
}
impl UnsharedReadmittedRelationalSection {
    pub fn prepare_close(
        self,
    ) -> Result<ClosingRelationalSection, (Self, RelationalSectionCloseDenial)> {
        match super::close::prepare_attached_close(
            self.attached,
            Some(self.recovered),
            Some(self.outcome),
        ) {
            Ok(closing) => Ok(closing),
            Err((attached, recovered, outcome, cause)) => Err((
                Self {
                    attached,
                    recovered: recovered.expect("same actual recovered authority"),
                    outcome: outcome.expect("same actual recovery outcome"),
                },
                cause,
            )),
        }
    }
    pub fn home_query_finalization_parts(
        &mut self,
    ) -> (
        &mut RelationalRuntime,
        &RecoveredRelationalRuntimeAuthority,
        &RecoveryOutcome,
    ) {
        (&mut self.attached.runtime, &self.recovered, &self.outcome)
    }
    pub fn share(self) -> RelationalStoreSection {
        self.attached
            .share(Some(self.recovered), Some(self.outcome))
    }
}
