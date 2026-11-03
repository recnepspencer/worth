//! Visit actual original Head/Full history registrations, never resident copies.
use super::*;
impl NativeRegionSourceRetirement {
    pub(super) fn advance_resident(
        &mut self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), NativeRegionSourceRetirementDenial> {
        if matches!(self.stage, Stage::SelectResident) {
            if self.resident_slot == OriginalResidentSlot::Complete {
                self.stage = Stage::Complete;
                return Ok(());
            }
            self.resident = self.cell.source_retirement_resident(self.resident_slot)?;
            match self.resident.as_ref() {
                None => self.resident_slot = self.resident_slot.next(),
                Some(original) => {
                    self.custody = original._history_custody.clone();
                    self.stage = Stage::RegionHistory;
                }
            }
            return Ok(());
        }
        if self.custody.is_none() {
            let original = self
                .resident
                .as_ref()
                .expect("same selected original resident");
            match self.stage {
                Stage::RegionHistory => {
                    self.finish_member();
                    return Ok(());
                }
                Stage::EntityPrefixes => {
                    if self.row == original.partition.entity_arena.history_prefixes.len() {
                        self.row = 0;
                        self.stage = Stage::RelationPrefixes;
                        return Ok(());
                    }
                    self.custody = original.partition.entity_arena.history_prefixes[self.row]
                        .as_ref()
                        .map(|prefix| prefix.custody.clone());
                }
                Stage::RelationPrefixes => {
                    if self.row == original.partition.relation_arena.history_prefixes.len() {
                        self.resident = None;
                        self.resident_slot = self.resident_slot.next();
                        self.stage = Stage::SelectResident;
                        return Ok(());
                    }
                    self.custody = original.partition.relation_arena.history_prefixes[self.row]
                        .as_ref()
                        .map(|prefix| prefix.custody.clone());
                }
                Stage::BackingSymbols
                | Stage::HeadSymbols
                | Stage::CellMember
                | Stage::SelectResident
                | Stage::Complete => unreachable!("selected history stage"),
            }
        }
        match self.custody.as_ref() {
            None => self.finish_member(),
            Some(custody) => {
                if self.pass.retires() {
                    if custody.loan_source_retirement(&mut self.handoff)? {
                        self.finish_member();
                    }
                } else {
                    custody.preflight_source_retirement(source)?;
                    self.finish_member();
                }
            }
        }
        Ok(())
    }
}
