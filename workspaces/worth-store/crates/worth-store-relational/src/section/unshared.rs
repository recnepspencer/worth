//! Concrete Home construction phases precede the Native owner's first Arc.
mod close;
use super::*;
mod source_attachment;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        FreshRelationalSourceAdmission, RecoveredRelationalRuntimeAuthority, RecoveryOutcome,
    },
    runtime::{
        RelationalInitialSchemaInstallationReceipt, RelationalRuntime,
        RelationalRuntimeConstructionDenial, RelationalRuntimeDefinition,
    },
};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, AnchoredRecordMembership, ServingPhysicalRuntime,
};

pub(super) struct AttachedRelationalRuntime {
    pub(super) runtime: RelationalRuntime,
    pub(super) cold_source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    pub(super) checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
}
pub struct UnrecoveredRelationalSection {
    attached: AttachedRelationalRuntime,
    prepared_source: Option<super::PreparedRelationalHomeSource>,
}
pub struct UnsharedReadmittedRelationalSection {
    attached: AttachedRelationalRuntime,
    source: super::readmission::source::ReadmittedRelationalSource,
}

impl AttachedRelationalRuntime {
    pub(super) fn attach(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        cold_source: Arc<crate::cold_source::StoreColdSource>,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
        transition: Option<worth_proof::HomeProgramTransitionAuthority>,
        transition_required: Option<worth_proof::HomeProgramTransitionRequired>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let medium = crate::journal_medium::StoreJournalMedium::new(
            serving.clone(),
            placement,
            cold_source.clone(),
            checkpoint_selection.clone(),
        )
        .with_home_transition_authority(transition, transition_required);
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
        source: Option<super::readmission::source::ReadmittedRelationalSource>,
    ) -> RelationalStoreSection {
        RelationalStoreSection {
            runtime: Arc::new(self.runtime),
            cold_source: self.cold_source,
            serving: self.serving,
            checkpoint_selection: self.checkpoint_selection,
            source,
        }
    }
}

impl UnrecoveredRelationalSection {
    pub fn home_fresh_schema_initialization(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
        worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
    > {
        use worth_relational::facade::{
            durability::RelationalInitialInputsRefusal as Input,
            runtime::RelationalInitialSchemaInstallationDenial as Denial,
        };
        request
            .in_scope(|_| {
                self.attached
                    .runtime
                    .begin_fresh_home_initialization(request)
            })
            .map_err(|cause| Denial::from_initial_inputs(Input::Request(cause.into())))?
            .map_err(Denial::from_initial_inputs)
    }
    pub(super) fn attach_with_source(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        source: Arc<crate::cold_source::StoreColdSource>,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
        transition: Option<worth_proof::HomeProgramTransitionAuthority>,
        transition_required: Option<worth_proof::HomeProgramTransitionRequired>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        AttachedRelationalRuntime::attach(
            definition,
            serving,
            placement,
            source,
            checkpoint_selection,
            transition,
            transition_required,
        )
        .map(|attached| Self {
            attached,
            prepared_source: None,
        })
    }
    pub fn home_schema_installation_runtime(&mut self) -> &mut RelationalRuntime {
        &mut self.attached.runtime
    }

    /// Closed Store Home routing supplies its actual physical fresh custody.
    /// Source shape is checked by Native; it never chooses this entry.
    pub fn admit_fresh_home_from_source(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<UnsharedReadmittedRelationalSection, (Self, RelationalSectionReadmissionDenial)>
    {
        match super::readmission::fresh_source::admit(
            &mut self.attached.runtime,
            &self.attached.cold_source,
            execution,
        ) {
            Ok(admission) => Ok(UnsharedReadmittedRelationalSection {
                attached: self.attached,
                source: super::readmission::source::ReadmittedRelationalSource::Fresh(admission),
            }),
            Err(cause) => Err((self, cause)),
        }
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
                source: super::readmission::source::ReadmittedRelationalSource::Recovered {
                    authority: admitted.authority,
                    outcome: admitted.outcome,
                },
            }),
            Err(cause) => Err((self, cause)),
        }
    }
}
impl UnsharedReadmittedRelationalSection {
    pub fn home_fresh_query_finalization_parts(
        &mut self,
    ) -> Result<
        (&mut RelationalRuntime, &FreshRelationalSourceAdmission),
        RelationalSectionReadmissionDenial,
    > {
        match &self.source {
            super::readmission::source::ReadmittedRelationalSource::Fresh(admission) => {
                Ok((&mut self.attached.runtime, admission))
            }
            super::readmission::source::ReadmittedRelationalSource::Recovered { .. } => {
                Err(RelationalSectionReadmissionDenial::FreshSourceAdmissionRequired)
            }
        }
    }
    pub fn home_recovered_query_finalization_parts(
        &mut self,
    ) -> Result<
        (
            &mut RelationalRuntime,
            &RecoveredRelationalRuntimeAuthority,
            &RecoveryOutcome,
        ),
        RelationalSectionReadmissionDenial,
    > {
        match &self.source {
            super::readmission::source::ReadmittedRelationalSource::Recovered {
                authority,
                outcome,
            } => Ok((&mut self.attached.runtime, authority, outcome)),
            super::readmission::source::ReadmittedRelationalSource::Fresh(_) => {
                Err(RelationalSectionReadmissionDenial::RecoveredSourceAdmissionRequired)
            }
        }
    }
    pub fn share(self) -> RelationalStoreSection {
        self.attached.share(Some(self.source))
    }
}
