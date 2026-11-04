//! Owner-directed source inventory, with constant current-owner storage.
mod checkpoints;
mod directories;
mod members;
use super::history::NativeHistorySourceRetirement;
use super::*;
use crate::durability::medium::NativeMemberRetirementHandoff;
use crate::identity::data::PartitionId;
use crate::runtime::RuntimeSymbolTable;
enum Stage {
    History,
    DeferredHistory,
    Edition,
    Mirror,
    Symbols,
    Nodes,
    Events,
    Index,
    IndexMember,
    IndexDirectory,
    SelectedDirectory,
    RetainedDirectory,
    RetainedCanonicalDirectory,
    Complete,
}
pub(super) struct NativeSourceGraphRetirement {
    pass: NativeSourcePass,
    stage: Stage,
    history: Option<NativeHistorySourceRetirement>,
    partition: Option<PartitionId>,
    cell: Option<NativeRegionSourceRetirement>,
    symbols: Option<RuntimeSymbolTable>,
    member: Option<NativeMemberRetirementHandoff>,
    directory: Option<NativeDirectoryRetirementHandoff>,
    checkpoint: usize,
}
impl NativeSourceGraphRetirement {
    pub(super) fn new(
        runtime: &RelationalRuntime,
        pass: NativeSourcePass,
    ) -> Result<Self, RelationalSourceCloseDenial> {
        let recovered = runtime
            .durability
            .source_root_inventory()
            .map_err(|()| RelationalSourceCloseDenial::Busy)?;
        Ok(Self {
            pass,
            stage: Stage::History,
            history: Some(NativeHistorySourceRetirement::new(
                runtime.history.source_retirement_binding(),
                pass,
                recovered,
            )),
            partition: None,
            cell: None,
            symbols: None,
            member: None,
            directory: None,
            checkpoint: 0,
        })
    }
    pub(super) fn advance(
        &mut self,
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<bool, RelationalSourceCloseDenial> {
        use RelationalSourceCloseDenial as Denial;
        if self.symbols.is_some() {
            self.advance_symbols(request, source)?;
            return Ok(false);
        }
        if let Some(cell) = &mut self.cell {
            if cell.advance(request, source)? == NativeRegionSourceRetirementProgress::Complete {
                self.cell = None;
            }
            return Ok(false);
        }
        if let Some(history) = &mut self.history {
            if history.advance(request, source)? {
                self.history = None;
            }
            return Ok(false);
        }
        match self.stage {
            Stage::History => {
                self.history = runtime
                    .indexes
                    .source_retirement_history()
                    .map_err(|()| Denial::Busy)?
                    .map(|original| NativeHistorySourceRetirement::new(original, self.pass, None));
                self.stage = Stage::DeferredHistory;
            }
            Stage::DeferredHistory => self.stage = Stage::Edition,
            Stage::Edition => match runtime
                .partitions
                .source_retirement_edition_cell(self.partition, self.pass.retires())
                .map_err(|()| Denial::Busy)?
            {
                Some((id, cell)) => {
                    self.partition = Some(id);
                    self.cell = Some(NativeRegionSourceRetirement::new(cell, self.pass));
                }
                None => {
                    self.partition = None;
                    self.stage = Stage::Mirror;
                }
            },
            Stage::Mirror => match runtime
                .partitions
                .source_retirement_target(self.partition, self.pass.retires())
                .map_err(|()| Denial::Busy)?
            {
                Some((id, cell, symbols)) => {
                    self.partition = Some(id);
                    self.symbols = symbols;
                    self.cell = Some(NativeRegionSourceRetirement::new(cell, self.pass));
                }
                None => self.stage = Stage::Symbols,
            },
            Stage::Symbols => {
                self.symbols = Some(runtime.services.symbols.clone());
                self.stage = Stage::Nodes;
            }
            Stage::Nodes | Stage::Events | Stage::IndexMember => {
                self.advance_member(runtime, request, source)?
            }
            Stage::Index => {
                if !self.pass.retires() {
                    runtime
                        .indexes
                        .preflight_source_retirement(source)
                        .map_err(Denial::Checkpoint)?;
                }
                self.stage = Stage::IndexMember;
            }
            Stage::IndexDirectory | Stage::SelectedDirectory => {
                self.advance_directory(runtime, request, source)?
            }
            Stage::RetainedDirectory | Stage::RetainedCanonicalDirectory => {
                self.advance_checkpoint_directory(runtime, request, source)?;
            }
            Stage::Complete => return Ok(true),
        }
        Ok(false)
    }
}
