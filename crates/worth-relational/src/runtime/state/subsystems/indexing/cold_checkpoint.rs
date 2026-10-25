//! A retained index image is admitted explicitly, before an operation uses it.
use super::super::RuntimeOwnedState;
use super::{IndexDefinitionReadBinding, IndexingState, IndexingSubsystem};
use crate::durability::data::RelationalCheckpointDirectory;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::runtime::HistorySubsystem;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(super) struct DeferredIndexImage {
    directory: Arc<RelationalCheckpointDirectory>,
    history: HistorySubsystem,
    admitted: Mutex<Option<Result<Arc<IndexingState>, RelationalStorageUnavailable>>>,
}

impl DeferredIndexImage {
    fn readmit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<IndexingState>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut admitted = self
            .admitted
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(result) = &*admitted {
            return result.clone().map_err(Into::into);
        }
        let refusal = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Index(
                self.directory
                    .manifest
                    .index_definitions
                    .first()
                    .map_or(crate::indexes::data::DerivedIndexId(0), |definition| {
                        definition.index_id
                    }),
            ),
            cause,
        };
        let result = (|| {
            let key = self.directory.manifest.derived_indexes;
            let bytes = self
                .directory
                .source
                .read_member(key, 1 << 30)
                .map_err(refusal)?;
            let payload =
                member_frame::decode(&bytes, key, MemberKind::DerivedIndex).map_err(refusal)?;
            let (legacy, checkpoint, format): (
                crate::indexes::data::DerivedIndexArtifacts,
                Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
                u16,
            ) = rmp_serde::from_slice(payload).map_err(|_| refusal(Cause::Damaged))?;
            if !crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::supports_outer_format(format, checkpoint.is_some()) {
                return Err(refusal(Cause::Damaged).into());
            }
            let mut state = IndexingState::default();
            for definition in &self.directory.manifest.index_definitions {
                state.insert_definition(definition.clone());
            }
            crate::durability::derived_index_artifacts::restore_checkpoint_derived_index_artifacts(
                &mut state,
                &self.history,
                &legacy,
                checkpoint.as_ref(),
                &mut Default::default(),
            )
            .map_err(|_| refusal(Cause::Damaged))?;
            crate::execution::check_request(execution).map_err(Failure::Request)?;
            Ok(Arc::new(state))
        })();
        match &result {
            Ok(state) => {
                *admitted = Some(Ok(state.clone()));
            }
            Err(Failure::StorageUnavailable(failure)) if failure.cause.is_proven_damage() => {
                *admitted = Some(Err(*failure));
            }
            Err(Failure::StorageUnavailable(_)) => {}
            Err(Failure::Request(_)) => {}
        }
        result
    }
}

impl IndexingSubsystem {
    pub(crate) fn install_cold_checkpoint(
        &self,
        directory: Arc<RelationalCheckpointDirectory>,
        history: &HistorySubsystem,
    ) {
        let mut state = IndexingState::default();
        for definition in &directory.manifest.index_definitions {
            state.insert_definition(definition.clone());
        }
        state.cold_checkpoint = Some(Arc::new(DeferredIndexImage {
            directory,
            history: history.detached_owner_snapshot(),
            admitted: Mutex::new(None),
        }));
        self.install(state);
    }
    pub(crate) fn carried_checkpoint_image(
        &self,
    ) -> Option<crate::durability::medium::RelationalDurableMemberLocator> {
        self.state
            .read()
            .cold_checkpoint
            .as_ref()
            .map(|image| image.directory.manifest.derived_indexes)
    }
}

impl IndexDefinitionReadBinding {
    pub(crate) fn demand_checkpoint_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        if self.state.read().definitions.is_empty() {
            return Ok(());
        }
        demand(&self.state, execution)
    }
}

fn demand(
    owner: &RuntimeOwnedState<IndexingState>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let Some(image) = owner.read().cold_checkpoint.clone() else {
        return Ok(());
    };
    let admitted = image.readmit(execution)?;
    let mut state = owner.write();
    if state
        .cold_checkpoint
        .as_ref()
        .is_some_and(|pending| Arc::ptr_eq(pending, &image))
    {
        for generation in admitted.generations.all() {
            state.restore_generation(generation.as_ref().clone());
        }
        state.cold_checkpoint = None;
    }
    Ok(())
}
