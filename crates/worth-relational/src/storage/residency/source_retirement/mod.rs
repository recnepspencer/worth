//! Constant current-owner custody through validation and physical retirement.
use super::{RelationalRegionResidency, ResidentRegion};
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial, RelationalDurableColdSource,
    RelationalDurableMemberRetirementRefusal,
};
use crate::runtime::RuntimeSymbolTable;
use crate::storage::substrate::RecordHistoryCustody;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod member;
mod resident;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::storage::residency) enum OriginalResidentSlot {
    Current,
    RawHead,
    RawFull,
    Complete,
}
impl OriginalResidentSlot {
    fn next(self) -> Self {
        match self {
            Self::Current => Self::RawHead,
            Self::RawHead => Self::RawFull,
            Self::RawFull | Self::Complete => Self::Complete,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum NativeSourcePass {
    Validate,
    Retire,
}
impl NativeSourcePass {
    pub(crate) fn retires(self) -> bool {
        matches!(self, Self::Retire)
    }
}
#[derive(Debug)]
pub(crate) enum NativeRegionSourceRetirementDenial {
    Busy,
    Member(RelationalDurableMemberRetirementRefusal),
    Execution(worth_execution::WorkCeilingDenial),
}
impl From<NativeMemberSlotDenial> for NativeRegionSourceRetirementDenial {
    fn from(cause: NativeMemberSlotDenial) -> Self {
        match cause {
            NativeMemberSlotDenial::Busy => Self::Busy,
            NativeMemberSlotDenial::Refused(cause) => Self::Member(cause),
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum NativeRegionSourceRetirementProgress {
    Advanced,
    Complete,
}
enum Stage {
    BackingSymbols,
    HeadSymbols,
    CellMember,
    SelectResident,
    RegionHistory,
    EntityPrefixes,
    RelationPrefixes,
    Complete,
}
pub(crate) struct NativeRegionSourceRetirement {
    cell: Arc<RelationalRegionResidency>,
    pass: NativeSourcePass,
    stage: Stage,
    symbols: Option<RuntimeSymbolTable>,
    resident: Option<Arc<ResidentRegion>>,
    resident_slot: OriginalResidentSlot,
    row: usize,
    custody: Option<Arc<RecordHistoryCustody>>,
    handoff: Option<NativeMemberRetirementHandoff>,
}
impl NativeRegionSourceRetirement {
    pub(crate) fn new(cell: Arc<RelationalRegionResidency>, pass: NativeSourcePass) -> Self {
        Self {
            cell,
            pass,
            stage: Stage::BackingSymbols,
            symbols: None,
            resident: None,
            resident_slot: OriginalResidentSlot::Current,
            row: 0,
            custody: None,
            handoff: None,
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<NativeRegionSourceRetirementProgress, NativeRegionSourceRetirementDenial> {
        use NativeRegionSourceRetirementProgress as Progress;
        if matches!(self.stage, Stage::Complete) {
            return Ok(Progress::Complete);
        }
        // A cached observation is returned before attempting another callback.
        if self.handoff.is_some() {
            return self.advance_handoff(request, source);
        }
        request
            .consult()
            .map_err(NativeRegionSourceRetirementDenial::Execution)?;
        match self.stage {
            Stage::BackingSymbols | Stage::HeadSymbols => {
                if self.symbols.is_none() {
                    self.symbols = self
                        .cell
                        .source_retirement_symbols(matches!(self.stage, Stage::BackingSymbols));
                }
                match &self.symbols {
                    None => self.finish_member(),
                    Some(symbols) => {
                        if self.pass.retires() {
                            if symbols.loan_source_retirement(&mut self.handoff)? {
                                self.finish_member();
                            }
                        } else {
                            symbols.preflight_source_retirement(source)?;
                            self.finish_member();
                        }
                    }
                }
            }
            Stage::CellMember => {
                if self.pass.retires() {
                    if self.cell.loan_source_member(&mut self.handoff)? {
                        self.finish_member();
                    }
                } else {
                    self.cell.preflight_source_member(source)?;
                    self.finish_member();
                }
            }
            Stage::SelectResident
            | Stage::RegionHistory
            | Stage::EntityPrefixes
            | Stage::RelationPrefixes => {
                self.advance_resident(source)?;
            }
            Stage::Complete => return Ok(Progress::Complete),
        }
        Ok(Progress::Advanced)
    }
    fn finish_member(&mut self) {
        self.symbols = None;
        self.custody = None;
        self.stage = match self.stage {
            Stage::BackingSymbols => Stage::HeadSymbols,
            Stage::HeadSymbols => Stage::CellMember,
            Stage::CellMember => Stage::SelectResident,
            Stage::RegionHistory => {
                self.row = 0;
                Stage::EntityPrefixes
            }
            Stage::EntityPrefixes => {
                self.row += 1;
                Stage::EntityPrefixes
            }
            Stage::RelationPrefixes => {
                self.row += 1;
                Stage::RelationPrefixes
            }
            Stage::SelectResident | Stage::Complete => {
                unreachable!("a real selected member finished")
            }
        };
    }
}
