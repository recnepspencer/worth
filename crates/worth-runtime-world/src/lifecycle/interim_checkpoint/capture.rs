//! Freeze only committed originals and their actual Native capture tokens.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;
use worth_relational::facade::durability::CapturedNativeInterimAdmission;

impl WorldInterimCheckpointPort {
    pub fn capture(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        native: &CapturedNativeInterimAdmission,
        application: CanonicalDigestId,
    ) -> Result<CapturedWorldInterimAdmission, Stop> {
        request_scope::run(request, || {
            request.consult().map_err(Stop::Request)?;
            let owner = self.owner.upgrade().ok_or(Stop::OwnerUnavailable)?;
            let _operation = owner.try_admit_call().ok_or(Stop::OwnerUnavailable)?;
            let state = &owner.state;
            if state.durable.is_some() || observation.owner_identity() != state.owner_identity {
                return Err(Stop::BindingMismatch);
            }
            let cell = state
                .branches
                .branch_cell(observation.branch_identity())
                .ok_or(Stop::OwnerUnavailable)?;
            if observation
                .mismatch_against_snapshot(&cell.atomic_snapshot())
                .is_some()
            {
                return Err(Stop::CaptureChanged);
            }
            let originals = state.history.snapshot_memory_originals(request)?;
            let port = state
                .relational
                .as_ref()
                .ok_or(Stop::OwnerUnavailable)?
                .basis_port();
            let component = port
                .observe_memory_component(request, observation.basis().relational_basis())
                .map_err(Stop::Component)?;
            let anchor_component = port
                .capture_interim_current_memory_component(request, native, &component)
                .map_err(Stop::Component)?;
            let anchor = reference(observation, request)?;
            let mut rows = ExecutionRequestArrayBuilder::allocate(*request, originals.len())
                .map_err(Stop::Backing)?;
            for original in originals.iter() {
                charge(request, 1)?;
                let mut row = CapturedRow {
                    meaning: original.meaning.clone(),
                    before: port
                        .capture_interim_memory_component(
                            request,
                            native,
                            &original.before_component,
                        )
                        .map_err(Stop::Component)?,
                    after: port
                        .capture_interim_memory_component(
                            request,
                            native,
                            &original.after_component,
                        )
                        .map_err(Stop::Component)?,
                    bound_to_anchor: original.matches_receiver(observation),
                    digest: CanonicalDigestId::new([0; 32]),
                };
                row.digest = wire::row_digest(&row, request)?;
                rows.push(row).map_err(Stop::Backing)?;
            }
            let rows = rows.seal().map_err(Stop::Backing)?;
            // Snapshot equality is checked before and after owner work, without
            // calling Native while the registry or branch cell is locked.
            if !state
                .history
                .matches_memory_originals(&originals, request)?
                || observation
                    .mismatch_against_snapshot(&cell.atomic_snapshot())
                    .is_some()
            {
                return Err(Stop::CaptureChanged);
            }
            CapturedRegistry::prepare(native, application, anchor, anchor_component, rows, request)
        })
    }
}
