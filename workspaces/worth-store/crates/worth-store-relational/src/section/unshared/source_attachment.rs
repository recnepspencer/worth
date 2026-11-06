//! Actual source preparation chooses fresh or existing Native attachment once.
use super::*;
impl UnrecoveredRelationalSection {
    pub fn attach_fresh_home(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
        prepared: crate::PreparedFreshRelationalHomeSource,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        Self::attach_with_source(
            definition,
            serving,
            placement,
            prepared.source,
            checkpoint_selection,
            prepared.transition,
            prepared.transition_required,
        )
    }
    pub fn attach_prepared_home(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        checkpoint_selection: Arc<dyn RelationalCheckpointSelectionSource>,
        prepared: crate::PreparedRelationalHomeSource,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        AttachedRelationalRuntime::attach(
            definition,
            serving,
            placement,
            prepared.source.clone(),
            checkpoint_selection,
            prepared.transition.clone(),
            prepared.transition_required.clone(),
        )
        .map(|attached| Self {
            attached,
            prepared_source: Some(prepared),
        })
    }
    pub fn readmit_prepared_home(
        mut self,
        request: &ExecutionRequest<'_, '_>,
        transition: Option<
            &worth_proof::AdmittedHomeProgramTransitionCatalog<
                worth_relational::facade::runtime::RelationalHomeProgramCatalogReadmissionSource<
                    '_,
                >,
            >,
        >,
    ) -> Result<
        (UnsharedReadmittedRelationalSection, Option<[u64; 2]>),
        (Self, RelationalSectionReadmissionDenial),
    > {
        let Some(prepared) = &self.prepared_source else {
            return Err((
                self,
                RelationalSectionReadmissionDenial::PreparedHomeSourceRequired,
            ));
        };
        match super::super::readmission::prepared_source::recover(
            &mut self.attached.runtime,
            &prepared.original,
            request,
            transition,
        ) {
            Ok(admitted) => Ok((
                UnsharedReadmittedRelationalSection {
                    attached: self.attached,
                    source:
                        super::super::readmission::source::ReadmittedRelationalSource::Recovered {
                            authority: admitted.authority,
                            outcome: admitted.outcome,
                        },
                },
                admitted.selected_coverage,
            )),
            Err(cause) => Err((self, cause)),
        }
    }
}
