//! Rebind captured originals to the real newly installed World receiver.
use super::*;
use crate::history::{MemoryPublicationReceiver, MemoryPublicationRecord};
use std::sync::atomic::AtomicBool;
use worth_relational::facade::durability::AdmittedNativeInterimReceiver;

impl WorldInterimCheckpointPort {
    pub fn restore(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        native: &AdmittedNativeInterimReceiver,
        native_origin: &CapturedNativeInterimOrigin,
        origin: &CapturedWorldInterimOrigin,
        continuation: Option<
            &worth_relational::facade::durability::NativeInterimBootstrapContinuation,
        >,
        application: CanonicalDigestId,
    ) -> Result<(), Stop> {
        request_scope::run(request, || {
            origin.validate_transport(origin.0.wrapper.bytes(), request)?;
            let captured = &origin.0.captured;
            if native_origin.coordinate() != captured.coordinate
                || native_origin.wrapper_digest() != origin.0.wrapper_digest
                || application != captured.application
            {
                return Err(Stop::BindingMismatch);
            }
            let owner = self.owner.upgrade().ok_or(Stop::OwnerUnavailable)?;
            let _operation = owner.try_admit_call().ok_or(Stop::OwnerUnavailable)?;
            let state = &owner.state;
            if state.durable.is_some() || state.owner_identity != observation.owner_identity() {
                return Err(Stop::BindingMismatch);
            }
            if captured.anchor.reference.branch() != observation.branch_identity().name().as_str() {
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
            let reservation = state.history.reserve_memory_readmission()?;
            let port = state
                .relational
                .as_ref()
                .ok_or(Stop::OwnerUnavailable)?
                .basis_port();
            let anchor = port
                .readmit_interim_memory_component(request, native, &captured.anchor_component)
                .map_err(Stop::Native)?;
            let actual = port
                .export_memory_component(request, &anchor)
                .map_err(Stop::Component)?;
            // An archived Native description proves an original only. This actual
            // product observation must still select the same Native target.
            if let Some(continuation) = continuation {
                port.install_interim_bootstrap_receiver(
                    request,
                    continuation,
                    native,
                    observation.basis().relational_basis(),
                )
                .map_err(Stop::Native)?;
            } else {
                let selected = port
                    .export_recorded_current_component(
                        request,
                        observation.basis().relational_basis(),
                    )
                    .map_err(Stop::Component)?;
                comparison_work(&actual, request)?;
                if actual != selected {
                    return Err(Stop::BindingMismatch);
                }
            }
            let receiver = reference(observation, request)?;
            let mut staged = worth_execution::ExecutionRequestArrayBuilder::allocate(
                *request,
                captured.rows.len(),
            )
            .map_err(Stop::Backing)?;
            for row in captured.rows.iter() {
                if wire::row_digest(row, request)? != row.digest {
                    return Err(Stop::AuthenticationMismatch);
                }
                let before = port
                    .readmit_interim_memory_component(request, native, &row.before)
                    .map_err(Stop::Native)?;
                let after = port
                    .readmit_interim_memory_component(request, native, &row.after)
                    .map_err(Stop::Native)?;
                let memory = reserve_arc::<MemoryPublicationRecord>(request, 0)?;
                charge(request, 1)?;
                let record = Arc::new(MemoryPublicationRecord {
                    meaning: row.meaning.clone(),
                    before_component: before,
                    after_component: after,
                    receiver: if row.bound_to_anchor {
                        MemoryPublicationReceiver::Restored(receiver.clone())
                    } else {
                        MemoryPublicationReceiver::Archived
                    },
                    committed: AtomicBool::new(true),
                    _funding: memory,
                });
                staged.push(record).map_err(Stop::Backing)?;
            }
            reservation.install(staged.seal().map_err(Stop::Backing)?, request)
        })
    }
}

fn comparison_work(
    value: &worth_relational::facade::branch::RecordedRelationalComponentReference,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Stop> {
    let descriptor = value.descriptor();
    let variable = descriptor
        .branch_id()
        .0
        .len()
        .checked_add(descriptor.reference().branch_id().as_str().len())
        .and_then(|n| {
            n.checked_add(
                descriptor
                    .reference()
                    .target()
                    .as_basis()
                    .map_or(0, |target| target.parent_commit_ids().len()),
            )
        })
        .and_then(|n| {
            n.checked_add(match value.target() {
                worth_relational::facade::branch::RecordedRelationalComponentTarget::Empty => 0,
                worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(
                    target,
                ) => target
                    .receipt()
                    .branch_id
                    .0
                    .len()
                    .checked_add(target.receipt().parents.len())?,
            })
        })
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::SizeOverflow)?;
    charge(request, variable)
}
