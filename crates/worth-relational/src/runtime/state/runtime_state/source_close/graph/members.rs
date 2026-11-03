//! A provider callback runs outside Native gates; its observation stays owned.
use super::*;
use crate::durability::medium::RelationalDurableMemberRetirementObservation as Observation;
impl NativeSourceGraphRetirement {
    pub(super) fn advance_symbols(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), RelationalSourceCloseDenial> {
        let symbols = self
            .symbols
            .as_ref()
            .expect("same selected original dictionary");
        if !self.pass.retires() {
            symbols.preflight_source_retirement(source)?;
            self.symbols = None;
        } else if let Some(handoff) = &mut self.member {
            handoff.observe(request, source);
            let observation = symbols.return_source_retirement(&mut self.member, source)?;
            if complete(observation)? {
                self.symbols = None;
            }
        } else if symbols.loan_source_retirement(&mut self.member)? {
            self.symbols = None;
        }
        Ok(())
    }
    pub(super) fn advance_member(
        &mut self,
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), RelationalSourceCloseDenial> {
        let done = if !self.pass.retires() {
            match self.stage {
                Stage::Nodes => runtime.lineage.preflight_node_source_retirement(source)?,
                Stage::Events => runtime.lineage.preflight_event_source_retirement(source)?,
                Stage::IndexMember => {} // The index preflight validates both original slots.
                Stage::History
                | Stage::DeferredHistory
                | Stage::Edition
                | Stage::Mirror
                | Stage::Symbols
                | Stage::Index
                | Stage::IndexDirectory
                | Stage::SelectedDirectory
                | Stage::RetainedDirectory
                | Stage::RetainedCanonicalDirectory
                | Stage::Complete => unreachable!("selected physical member"),
            }
            true
        } else if let Some(handoff) = &mut self.member {
            handoff.observe(request, source);
            let observation = match self.stage {
                Stage::Nodes => runtime
                    .lineage
                    .return_node_source_retirement(&mut self.member, source)?,
                Stage::Events => runtime
                    .lineage
                    .return_event_source_retirement(&mut self.member, source)?,
                Stage::IndexMember => runtime
                    .indexes
                    .return_source_member(&mut self.member, source)?,
                Stage::History
                | Stage::DeferredHistory
                | Stage::Edition
                | Stage::Mirror
                | Stage::Symbols
                | Stage::Index
                | Stage::IndexDirectory
                | Stage::SelectedDirectory
                | Stage::RetainedDirectory
                | Stage::RetainedCanonicalDirectory
                | Stage::Complete => unreachable!("selected original member loan"),
            };
            complete(observation)?
        } else {
            match self.stage {
                Stage::Nodes => runtime
                    .lineage
                    .loan_node_source_retirement(&mut self.member)?,
                Stage::Events => runtime
                    .lineage
                    .loan_event_source_retirement(&mut self.member)?,
                Stage::IndexMember => runtime.indexes.loan_source_member(&mut self.member)?,
                Stage::History
                | Stage::DeferredHistory
                | Stage::Edition
                | Stage::Mirror
                | Stage::Symbols
                | Stage::Index
                | Stage::IndexDirectory
                | Stage::SelectedDirectory
                | Stage::RetainedDirectory
                | Stage::RetainedCanonicalDirectory
                | Stage::Complete => unreachable!("selected original member slot"),
            }
        };
        if done {
            self.stage = match self.stage {
                Stage::Nodes => Stage::Events,
                Stage::Events => Stage::Index,
                Stage::IndexMember => Stage::IndexDirectory,
                Stage::History
                | Stage::DeferredHistory
                | Stage::Edition
                | Stage::Mirror
                | Stage::Symbols
                | Stage::Index
                | Stage::IndexDirectory
                | Stage::SelectedDirectory
                | Stage::RetainedDirectory
                | Stage::RetainedCanonicalDirectory
                | Stage::Complete => unreachable!("completed physical member stage"),
            };
        }
        Ok(())
    }
}
fn complete(observation: Observation) -> Result<bool, RelationalSourceCloseDenial> {
    match observation {
        Observation::Complete => Ok(true),
        Observation::Advanced => Ok(false),
        Observation::Busy => Err(RelationalSourceCloseDenial::Busy),
        Observation::Stopped(cause) => Err(RelationalSourceCloseDenial::Execution(cause)),
        Observation::Refused(cause) => Err(RelationalSourceCloseDenial::Member(cause)),
    }
}
