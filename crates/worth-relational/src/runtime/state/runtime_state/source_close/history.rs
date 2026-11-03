//! Actual live/retired roots and canonical owners, one current path at a time.
use super::*;
use crate::branch::{
    RelationalRegionSourceWalk, RelationalRegionSourceWalkProgress as Walk,
    RelationalSourceRootSlot,
};
use crate::history::{data::CommitId, RelationalCommitEnvelopeResidency};
use crate::runtime::state::subsystems::{
    RelationalPerformedSourceWalk, RelationalSourceHistoryBinding,
};
enum Stage {
    LiveRoots,
    RetiredRoots,
    RecoveredRoots,
    Performed,
    CheckpointCatalog,
    Ledger,
    Heads,
    Complete,
}
pub(super) struct NativeHistorySourceRetirement {
    original: RelationalSourceHistoryBinding,
    recovered: Option<Arc<crate::durability::RecoveredRootInventory>>,
    recovered_after: Option<CommitId>,
    pass: NativeSourcePass,
    stage: Stage,
    live: usize,
    retired: Option<usize>,
    root: Option<Arc<crate::branch::RelationalBranchRoot>>,
    regions: Option<RelationalRegionSourceWalk>,
    cell: Option<NativeRegionSourceRetirement>,
    performed: RelationalPerformedSourceWalk,
    commit: Option<CommitId>,
    envelope: Option<Arc<RelationalCommitEnvelopeResidency>>,
    handoff: Option<NativeDirectoryRetirementHandoff>,
}
impl NativeHistorySourceRetirement {
    pub(super) fn new(
        original: RelationalSourceHistoryBinding,
        pass: NativeSourcePass,
        recovered: Option<Arc<crate::durability::RecoveredRootInventory>>,
    ) -> Self {
        let performed = original.performed_walk();
        Self {
            original,
            recovered,
            recovered_after: None,
            pass,
            stage: Stage::LiveRoots,
            live: 0,
            retired: None,
            root: None,
            regions: None,
            cell: None,
            performed,
            commit: None,
            envelope: None,
            handoff: None,
        }
    }
    pub(super) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<bool, RelationalSourceCloseDenial> {
        if let Some(cell) = &mut self.cell {
            if cell.advance(request, source)? == NativeRegionSourceRetirementProgress::Complete {
                self.cell = None;
            }
            return Ok(false);
        }
        if let Some(regions) = &mut self.regions {
            match regions.advance(self.pass.retires()) {
                Walk::Advanced => {}
                Walk::Region(_, cell) => {
                    self.cell = Some(NativeRegionSourceRetirement::new(cell, self.pass))
                }
                Walk::Complete => {
                    self.regions = None;
                    self.root = None;
                }
            }
            return Ok(false);
        }
        if self.envelope.is_some() {
            return self.advance_envelope(request, source);
        }
        match self.stage {
            Stage::LiveRoots => match self
                .original
                .live_root(self.live)
                .map_err(|()| RelationalSourceCloseDenial::Busy)?
            {
                RelationalSourceRootSlot::Root(root) => {
                    self.live += 1;
                    self.select_root(root);
                }
                RelationalSourceRootSlot::Empty => self.live += 1,
                RelationalSourceRootSlot::Complete => self.stage = Stage::RetiredRoots,
            },
            Stage::RetiredRoots => match self
                .original
                .retired_root(self.retired)
                .map_err(|()| RelationalSourceCloseDenial::Busy)?
            {
                Some((key, root)) => {
                    self.retired = Some(key);
                    self.select_root(root);
                }
                None => self.stage = Stage::RecoveredRoots,
            },
            Stage::RecoveredRoots => {
                let next = self
                    .recovered
                    .as_ref()
                    .map(|roots| roots.source_retirement_root(self.recovered_after))
                    .transpose()
                    .map_err(|()| RelationalSourceCloseDenial::Busy)?
                    .flatten();
                match next {
                    Some((commit, root)) => {
                        self.recovered_after = Some(commit);
                        if let Some(root) = root {
                            self.select_root(root);
                        }
                    }
                    None => self.stage = Stage::Performed,
                }
            }
            Stage::Performed => match self.performed.next() {
                Some(cell) => self.envelope = Some(cell),
                None => self.stage = Stage::CheckpointCatalog,
            },
            Stage::CheckpointCatalog => match self
                .original
                .checkpoint_envelope(self.commit)
                .map_err(|()| RelationalSourceCloseDenial::Busy)?
            {
                Some((id, cell)) => {
                    self.commit = Some(id);
                    self.envelope = Some(cell);
                }
                None => {
                    self.commit = None;
                    self.stage = Stage::Ledger;
                }
            },
            Stage::Ledger => match self
                .original
                .ledger_envelope(self.commit)
                .map_err(|()| RelationalSourceCloseDenial::Busy)?
            {
                Some((id, cell)) => {
                    self.commit = Some(id);
                    self.envelope = Some(cell);
                }
                None => {
                    self.live = 0;
                    self.stage = Stage::Heads;
                }
            },
            Stage::Heads => {
                if !self.pass.retires() {
                    self.stage = Stage::Complete;
                } else {
                    match self
                        .original
                        .retire_head(self.live)
                        .map_err(|()| RelationalSourceCloseDenial::Busy)?
                    {
                        None => {
                            return Err(RelationalSourceCloseDenial::Native(
                                RelationalRuntimeCloseDenial::ReadCustodyRetained,
                            ))
                        }
                        Some(false) => self.live += 1,
                        Some(true) => {
                            if self.original.external_custody_retained() {
                                return Err(RelationalSourceCloseDenial::Native(
                                    RelationalRuntimeCloseDenial::ReadCustodyRetained,
                                ));
                            }
                            self.stage = Stage::Complete;
                        }
                    }
                }
            }
            Stage::Complete => return Ok(true),
        }
        Ok(false)
    }
    fn select_root(&mut self, root: Arc<crate::branch::RelationalBranchRoot>) {
        self.regions = Some(root.source_retirement_regions());
        self.root = Some(root);
    }
    fn advance_envelope(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<bool, RelationalSourceCloseDenial> {
        let original = self.envelope.as_ref().expect("current original envelope");
        if !self.pass.retires() {
            original
                .preflight_source_retirement(source)
                .map_err(RelationalSourceCloseDenial::Checkpoint)?;
            self.envelope = None;
        } else if let Some(handoff) = &mut self.handoff {
            handoff.observe(request);
            if original
                .return_source_retirement(&mut self.handoff)
                .map_err(RelationalSourceCloseDenial::Checkpoint)?
            {
                self.envelope = None;
            }
        } else {
            if original
                .loan_source_retirement(source, &mut self.handoff)
                .map_err(RelationalSourceCloseDenial::Checkpoint)?
            {
                self.envelope = None;
            }
        }
        Ok(false)
    }
}
