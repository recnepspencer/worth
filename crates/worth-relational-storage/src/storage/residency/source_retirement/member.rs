//! A loan's task and callback observation remain in the same original cursor.
use super::*;
use crate::durability::medium::RelationalDurableMemberRetirementObservation as Observation;
impl NativeRegionSourceRetirement {
    pub(super) fn advance_handoff(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<NativeRegionSourceRetirementProgress, NativeRegionSourceRetirementDenial> {
        self.handoff
            .as_mut()
            .expect("same loaned original task")
            .observe(request, source);
        let observation = match self.stage {
            Stage::BackingSymbols | Stage::HeadSymbols => self
                .symbols
                .as_ref()
                .expect("same original dictionary")
                .return_source_retirement(&mut self.handoff, source)?,
            Stage::CellMember => self.cell.return_source_member(&mut self.handoff, source)?,
            Stage::RegionHistory | Stage::EntityPrefixes | Stage::RelationPrefixes => self
                .custody
                .as_ref()
                .expect("same original history registration")
                .return_source_retirement(&mut self.handoff, source)?,
            Stage::SelectResident | Stage::Complete => {
                unreachable!("only an original selected member loans custody")
            }
        };
        match observation {
            Observation::Advanced => {}
            Observation::Complete => self.finish_member(),
            Observation::Busy => return Err(NativeRegionSourceRetirementDenial::Busy),
            Observation::Stopped(cause) => {
                return Err(NativeRegionSourceRetirementDenial::Execution(cause))
            }
            Observation::Refused(cause) => {
                return Err(NativeRegionSourceRetirementDenial::Member(cause))
            }
        }
        Ok(NativeRegionSourceRetirementProgress::Advanced)
    }
}
