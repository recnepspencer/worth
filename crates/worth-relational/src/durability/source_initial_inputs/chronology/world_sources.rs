//! Released originals select exact predecessors; other fates cannot rewind Native.
use super::*;
use crate::durability::data::DurableBranchRootSchemaImage;
use crate::durability::{
    RelationalWorldOriginalDescription as Original, RelationalWorldSourceCatalog as Catalog,
    RelationalWorldSourceDescription as Source,
};
use crate::history::{data::CommitId, RelationalCommitIndexEntry as Index};
use worth_foundational::{CanonicalDigestId, WorldOriginalFate as Fate};
mod source;

pub(super) struct WorldSources<'a> {
    catalog: Option<&'a Catalog>,
    callers: ExecutionOrderedMap<CanonicalDigestId, &'a Original>,
    schemas: ExecutionOrderedMap<[u8; 32], [u8; 32]>,
    performed: ExecutionOrderedMap<CommitId, Performed<'a>>,
}
#[derive(Clone)]
struct Performed<'a> {
    branch: &'a BranchId,
    before: Axes<'a>,
    after: Axes<'a>,
    fate: Fate,
}
impl<'a> WorldSources<'a> {
    pub(super) fn prepare(
        catalog: Option<&'a Catalog>,
        schemas: &[DurableBranchRootSchemaImage],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut result = Self {
            catalog,
            callers: Default::default(),
            schemas: Default::default(),
            performed: Default::default(),
        };
        if catalog.is_none() {
            return Ok(result);
        }
        for schema in schemas {
            result
                .schemas
                .insert_cloned_native_for_request(
                    &schema.carrier_digest(),
                    &schema.schema_root(),
                    *request,
                )
                .map_err(map)?;
        }
        if let Some(catalog) = catalog {
            for row in catalog.originals() {
                if result.callers.contains_key(row.caller()) {
                    return Err(Refusal::Malformed);
                }
                result
                    .callers
                    .insert_cloned_native_for_request(row.caller(), &row, *request)
                    .map_err(map)?;
            }
        }
        Ok(result)
    }
    fn original(&self, index: &Index) -> Result<Option<&'a Original>, Refusal> {
        if self.catalog.is_none() {
            return Ok(None);
        }
        use sha2::{Digest, Sha256};
        let Some(caller) = index.association_discovery.association() else {
            return Ok(None);
        };
        if worth_foundational::HomeProgramTransitionCaller::recognizes(caller.as_bytes()) {
            return Ok(None);
        }
        let key = CanonicalDigestId::new(Sha256::digest(caller.as_bytes()).into());
        Ok(self.callers.get(&key).copied())
    }
    pub(super) fn before_canonical(
        &self,
        fold: &mut Fold<'a>,
        index: &'a Index,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Axes<'a>, Refusal> {
        let branch = &index.commit.branch_id;
        let current = fold
            .current
            .get(&branch)
            .cloned()
            .ok_or(Refusal::Malformed)?;
        let Some(original) = self.original(index)? else {
            return Ok(current);
        };
        let (source_branch, wanted) = self.source(fold, original.source())?;
        if source_branch != branch {
            return Err(Refusal::Malformed);
        }
        if !current.same_state(&wanted) {
            self.require_released_predecessor(branch, &current, &wanted)?;
            fold.record(branch, wanted.clone(), request)?;
        }
        Ok(wanted)
    }
    pub(super) fn retain(
        &mut self,
        index: &'a Index,
        before: Axes<'a>,
        after: Axes<'a>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let Some(original) = self.original(index)? else {
            return Ok(());
        };
        match original.effect() {
            Some(effect) => {
                if effect.commit() != index.commit.commit_id
                    || effect.position() != index.position
                    || effect.digest().bytes() != &index.payload_digest
                {
                    return Err(Refusal::Malformed);
                }
                match original.fate() {
                    Fate::Released => {
                        if original.receipt() != Some(&index.commit) {
                            return Err(malformed(
                                "World release lacks its exact original Native receipt",
                            ));
                        }
                    }
                    Fate::Open | Fate::Continued | Fate::Published => {
                        if original
                            .receipt()
                            .is_some_and(|receipt| receipt != &index.commit)
                        {
                            return Err(malformed(
                                "World original receipt disagrees with its canonical input",
                            ));
                        }
                    }
                }
            }
            None => match original.fate() {
                Fate::Open | Fate::Continued => {}
                Fate::Released | Fate::Published => return Err(Refusal::Malformed),
            },
        }
        if self.performed.contains_key(&index.commit.commit_id) {
            return Err(Refusal::Malformed);
        }
        let performed = Performed {
            branch: &index.commit.branch_id,
            before,
            after,
            fate: original.fate(),
        };
        self.performed
            .insert_cloned_native_for_request(&index.commit.commit_id, &performed, *request)
            .map_err(map)
    }
    fn require_released_predecessor(
        &self,
        branch: &BranchId,
        current: &Axes<'a>,
        wanted: &Axes<'a>,
    ) -> Result<(), Refusal> {
        let id = current.commit.ok_or(Refusal::Malformed)?.commit.commit_id;
        let original = self.performed.get(&id).ok_or(Refusal::Malformed)?;
        match original.fate {
            Fate::Released => {}
            Fate::Open | Fate::Continued | Fate::Published => return Err(Refusal::Malformed),
        }
        if original.branch != branch
            || !current.same_state(&original.after)
            || !wanted.same_state(&original.before)
        {
            return Err(Refusal::Malformed);
        }
        Ok(())
    }
    pub(super) fn before_transition(
        &self,
        fold: &mut Fold<'a>,
        entry: &'a TransitionOriginalRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let branch = match &entry.transition {
            Transition::Fork { source, .. } => source,
            Transition::Archive { .. }
            | Transition::DeletionRequested { .. }
            | Transition::Delete { .. } => entry.transition.branch(),
            Transition::RetainedFork { .. } => return Ok(()),
        };
        let current = fold.current.get(&branch).ok_or(Refusal::Malformed)?;
        if current.requires_transition(entry) {
            return Ok(());
        }
        let id = current.commit.ok_or(Refusal::Malformed)?.commit.commit_id;
        let performed = self.performed.get(&id).ok_or(Refusal::Malformed)?;
        self.require_released_predecessor(branch, current, &performed.before)?;
        if !performed.before.requires_transition(entry) {
            return Err(Refusal::Malformed);
        }
        fold.record(branch, performed.before.clone(), request)
    }
    pub(super) fn finish(
        &self,
        fold: &mut Fold<'a>,
        cells: &'a [crate::branch::RelationalBranchCellCheckpoint],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        for cell in cells {
            let branch = &cell.branch_id;
            let current = fold.current.get(&branch).ok_or(Refusal::Malformed)?;
            if matches_cell(current, cell) {
                continue;
            }
            let id = current.commit.ok_or(Refusal::Malformed)?.commit.commit_id;
            let performed = self.performed.get(&id).ok_or(Refusal::Malformed)?;
            // A checkpoint may capture the original successor before the next
            // open restores its released predecessor. Its cell chooses neither
            // an arbitrary old root nor a different original's predecessor.
            self.require_released_predecessor(branch, current, &performed.before)?;
            if !matches_cell(&performed.before, cell) {
                return Err(malformed(
                    "captured Native cell is not its released original predecessor",
                ));
            }
            fold.record(branch, performed.before.clone(), request)?;
        }
        Ok(())
    }
}
fn matches_cell(state: &Axes<'_>, cell: &crate::branch::RelationalBranchCellCheckpoint) -> bool {
    !state.removed
        && state.generation == cell.observation.generation()
        && state.truth == cell.truth_version
        && state.matches_target(cell.observation.target())
}
fn malformed(message: &str) -> Refusal {
    Refusal::Native(
        crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
            message,
        )
        .into(),
    )
}
