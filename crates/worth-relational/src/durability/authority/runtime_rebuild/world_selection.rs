//! Select World predecessors only from exact terminally released Native effects.
use super::root_inventory::RecoveredRootInventory;
use crate::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use crate::branch::recorded_basis::RecordedRelationalComponentReference as Recorded;
use crate::branch::reference::world_selection::WorldSourceSelection;
use crate::branch::root::RelationalBranchRoot;
use crate::durability::data::{RecoveryPlan, RelationalRecoveryStop as Stop};
use crate::durability::home_world_catalog::{
    RelationalWorldSourceCatalog, RelationalWorldSourceDescription as Source,
};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::{CanonicalDigestId, WorldOriginalFate};
use worth_proof::VerifiedHomeWorldSelection;
mod sources;
mod transitions;

pub(super) struct WorldReplayInputs<'a> {
    pub(super) history: &'a crate::runtime::state::subsystems::history::HistorySubsystem,
    pub(super) registry: &'a crate::schema::data::RelationalSchemaRegistry,
    pub(super) symbols: &'a crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub(super) checkpoint: &'a crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    pub(super) runtime_instance_id: u64,
    pub(super) visibility: crate::branch::head_visibility_inputs::BranchHeadVisibilityInputs<'a>,
}

pub(super) struct WorldSelectionReplay {
    admitted: Arc<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>,
    callers: ExecutionOrderedMap<CanonicalDigestId, usize>,
    effects: ExecutionOrderedMap<crate::history::data::CommitId, usize>,
    tails: ExecutionOrderedMap<crate::history::data::CommitId, usize>,
}
impl WorldSelectionReplay {
    pub(super) fn prepare(
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, Stop> {
        let Some(admitted) = &plan.home_world_selection else {
            return Ok(None);
        };
        let mut result = Self {
            admitted: admitted.clone(),
            callers: Default::default(),
            effects: Default::default(),
            tails: Default::default(),
        };
        for (index, row) in admitted.source().originals().iter().enumerate() {
            if result.callers.contains_key(row.caller()) {
                return Err(corrupt("duplicate World original caller"));
            }
            result
                .callers
                .insert_cloned_native_for_request(row.caller(), &index, *request)
                .map_err(ordered)?;
            if let Some(effect) = row.effect() {
                if result.effects.contains_key(&effect.commit()) {
                    return Err(corrupt("duplicate World Native effect"));
                }
                result
                    .effects
                    .insert_cloned_native_for_request(&effect.commit(), &index, *request)
                    .map_err(ordered)?;
            }
        }
        for (index, original) in plan.tail_log.iter().enumerate() {
            result
                .tails
                .insert_cloned_native_for_request(&original.commit_id(), &index, *request)
                .map_err(ordered)?;
        }
        Ok(Some(result))
    }
    pub(super) fn before(
        &self,
        request: &ExecutionRequest<'_, '_>,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &mut RecoveredRootInventory,
        pending: &crate::durability::migration::ReadmittedCanonicalCommit,
    ) -> Result<(), Stop> {
        let Some(caller) = pending.envelope().caller_association() else {
            return Ok(());
        };
        let caller = caller_digest(caller.as_bytes())?;
        let Some(index) = self.callers.get(&caller) else {
            return Ok(());
        };
        self.select(
            request,
            inputs,
            plan,
            roots,
            self.admitted.source().originals()[*index].source(),
            false,
        )
    }
    pub(super) fn finish(
        &self,
        request: &ExecutionRequest<'_, '_>,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &mut RecoveredRootInventory,
    ) -> Result<Arc<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>, Stop> {
        for source in self.admitted.source().heads() {
            self.select(request, inputs, plan, roots, source, true)?;
        }
        // Every non-disposed original product predecessor is needed by later
        // full World reconciliation, even when it stopped being a Native head.
        // Predecessors explicitly marked disposed in World's authoritative
        // `HistoryRetired` journal may already have been pruned from Native's
        // checkpoint while Track H released originals or horizon commits remain.
        for row in self.admitted.source().originals() {
            let recorded = if row.source_disposed() {
                let Some(recorded) =
                    self.recorded_source_optional(row.source(), inputs, plan, roots, request)?
                else {
                    continue;
                };
                recorded
            } else {
                self.recorded_source(row.source(), inputs, plan, roots, request)?
            };
            let Some((root, _)) = roots.exact_image(
                request,
                inputs.history,
                inputs.registry,
                inputs.symbols,
                &recorded,
            )?
            else {
                if row.source_disposed() {
                    self.require_present_recorded_entry(&recorded, inputs, plan, request)?;
                    continue;
                }
                return Err(corrupt("World handoff source image is unavailable"));
            };
            self.require_source(&recorded, &root, inputs, plan, request)?;
            roots.retain_world_selection_root(root.commit_id(), root, request)?;
        }
        for source in self.admitted.source().heads() {
            let recorded = self.recorded_source(source, inputs, plan, roots, request)?;
            let (root, _) = roots
                .exact_image(
                    request,
                    inputs.history,
                    inputs.registry,
                    inputs.symbols,
                    &recorded,
                )?
                .ok_or_else(|| corrupt("World handoff source image is unavailable"))?;
            roots.retain_world_selection_root(root.commit_id(), root, request)?;
        }
        roots.install_world_selection_handoff(self.admitted.authority().clone());
        Ok(self.admitted.clone())
    }
    fn select(
        &self,
        request: &ExecutionRequest<'_, '_>,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &mut RecoveredRootInventory,
        source: &Source,
        final_selection: bool,
    ) -> Result<(), Stop> {
        let recorded = self.recorded_source(source, inputs, plan, roots, request)?;
        let (root, descriptor) = roots
            .exact_image(
                request,
                inputs.history,
                inputs.registry,
                inputs.symbols,
                &recorded,
            )?
            .ok_or_else(|| corrupt("World selected source image is unavailable"))?;
        self.require_source(&recorded, &root, inputs, plan, request)?;
        let cell = inputs
            .history
            .branch_cell(descriptor.branch_id())
            .ok_or_else(|| corrupt("World selected branch is unavailable"))?;
        let previous = cell
            .root()
            .ok_or_else(|| corrupt("World selected branch has no head"))?;
        let current = crate::branch::basis_admission::descriptor_for_cell(&cell, &previous)
            .map_err(|_| corrupt("World selected head is not describable"))?;
        if current == descriptor && Arc::ptr_eq(&root, &previous) {
            return Ok(());
        }
        let commit = previous
            .commit_id()
            .ok_or_else(|| corrupt("World cannot displace an unaccounted Empty head"))?;
        let entry = self.entry(commit, inputs, plan, request)?;
        let caller = match entry.association_discovery.association() {
            Some(caller) => caller,
            None if final_selection => {
                // Native-only publications are not World originals. Preserve
                // their actual head; World still admits its own retained image.
                roots.retain_world_selection_root(root.commit_id(), root, request)?;
                return Ok(());
            }
            None => return Err(corrupt("World cannot displace an unassociated Native head")),
        };
        let caller_key = caller_digest(caller.as_bytes())?;
        let index = self
            .callers
            .get(&caller_key)
            .ok_or_else(|| corrupt("World head is not an original effect"))?;
        let row = &self.admitted.source().originals()[*index];
        match row.fate() {
            WorldOriginalFate::Released => {}
            WorldOriginalFate::Open | WorldOriginalFate::Continued if final_selection => {
                // A genuine unresolved original must reach World custody so its
                // caller can continue or release it. It never permits a new edit.
                roots.retain_world_selection_root(root.commit_id(), root, request)?;
                roots.retain_world_selection_root(Some(commit), previous, request)?;
                return Ok(());
            }
            WorldOriginalFate::Open
            | WorldOriginalFate::Continued
            | WorldOriginalFate::Published => {
                return Err(corrupt("World cannot displace a non-Released original"))
            }
        }
        if row.source() != source {
            return Err(corrupt("World release selected a different predecessor"));
        }
        let coordinates = row
            .effect()
            .ok_or_else(|| corrupt("World release has no Native coordinates"))?;
        if row.receipt() != Some(&entry.commit)
            || coordinates.position() != entry.position
            || coordinates.digest().bytes() != &entry.payload_digest
            || previous.descriptor() != Some(&entry.root)
            || previous.commit_evidence().map(|e| e.receipt()) != Some(&entry.commit)
        {
            return Err(corrupt(
                "World release does not match the actual Native effect",
            ));
        }
        let caller = entry
            .association_discovery
            .association()
            .ok_or_else(|| corrupt("World released effect has no original caller"))?;
        if worth_foundational::HomeProgramTransitionCaller::recognizes(caller.as_bytes())
            || caller_digest(caller.as_bytes())? != *row.caller()
        {
            return Err(corrupt(
                "World release cannot authorize a different or Home-step caller",
            ));
        }
        self.require_release_axes(&previous, &current, &descriptor, &root, inputs, request)?;
        // Admit staging custody before changing the actual selected cell. Later
        // World reconciliation must still reach this exact displaced image.
        roots.retain_world_selection_root(Some(commit), previous.clone(), request)?;
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        cell.select_recovered_world_source(
            WorldSourceSelection {
                history: inputs.history,
                visibility: inputs.visibility,
            },
            &descriptor,
            root,
        )?;
        Ok(())
    }
    fn require_release_axes(
        &self,
        previous: &Arc<RelationalBranchRoot>,
        current: &RelationalBranchBasisDescriptor,
        selected: &RelationalBranchBasisDescriptor,
        selected_root: &Arc<RelationalBranchRoot>,
        inputs: &WorldReplayInputs<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        match previous
            .commit_evidence()
            .ok_or_else(|| corrupt("released Native effect has no source evidence"))?
        {
            crate::branch::root::RelationalRootCommitEvidence::Original(original) => {
                let checkpoint = original
                    .envelope()
                    .branch_cell_checkpoint
                    .clone()
                    .ok_or_else(|| corrupt("released Native original has no predecessor cell"))?;
                let predecessor =
                    crate::branch::reference::RelationalBranchReferenceCell::from_checkpoint_with_root(
                        inputs.runtime_instance_id,
                        checkpoint,
                        Some(selected_root.clone()),
                    )
                    .map_err(|_| corrupt("released Native original predecessor is invalid"))?;
                let actual = crate::branch::basis_admission::descriptor_for_cell(
                    &predecessor,
                    selected_root,
                )
                .map_err(|_| corrupt("released Native predecessor cannot be described"))?;
                if &actual != selected {
                    return Err(corrupt("released Native original predecessor mismatch"));
                }
            }
            crate::branch::root::RelationalRootCommitEvidence::Checkpoint(commit) => {
                commit.require_publication_axes(selected, current, request)?;
            }
        }
        Ok(())
    }
}
fn caller_digest(bytes: &[u8]) -> Result<CanonicalDigestId, Stop> {
    use sha2::{Digest, Sha256};
    let bytes = if worth_foundational::HomeProgramTransitionCaller::recognizes(bytes) {
        worth_foundational::HomeProgramTransitionCaller::read(bytes)
            .map_err(|_| corrupt("malformed Native Home-step caller"))?
            .original()
    } else {
        bytes
    };
    Ok(CanonicalDigestId::new(Sha256::digest(bytes).into()))
}
fn corrupt(message: &str) -> Stop {
    crate::durability::data::DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        message,
    )
    .into()
}
pub(super) fn ordered(cause: worth_execution::ExecutionOrderedMapDenial) -> Stop {
    match cause {
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => Stop::Request(cause.into()),
        worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => Stop::Allocation(cause),
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
            corrupt("World source catalog capacity overflow")
        }
    }
}
