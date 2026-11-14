//! Every escaping decoded publication retains the actual grouped allocation bill.
use super::*;
use crate::history::data::CanonicalCoupledAuthoredSources;
use std::sync::Arc;

impl<Symbols> CanonicalEnvelopeWire<Symbols> {
    pub(super) fn attach_cascade_memory(&mut self) -> Result<(), &'static str> {
        let memory = self.readmission_memory.cascades.clone();
        if let Some(event) = &mut self.producer_cascade_retirement {
            let event = Arc::get_mut(event).ok_or("decoded producer cascade already aliased")?;
            event._decoded_memory = memory.clone();
            attach_origins(&mut event.publications, &memory)?;
        }
        if let Some(event) = &mut self.authored_cascade_retirement {
            let event = Arc::get_mut(event).ok_or("decoded authored cascade already aliased")?;
            event._decoded_memory = memory.clone();
            attach_origins(&mut event.publications, &memory)?;
        }
        if let Some(event) = &mut self.authored_cascade_restoration {
            let event = Arc::get_mut(event).ok_or("decoded restoration already aliased")?;
            event._decoded_memory = memory.clone();
            attach_origins(&mut event.publications, &memory)?;
        }
        if let Some(event) = &mut self.historical_output_selection {
            let event =
                Arc::get_mut(event).ok_or("decoded historical selection already aliased")?;
            event._decoded_memory = memory.clone();
            attach_origins(&mut event.publications, &memory)?;
        }
        if let Some(event) = &mut self.coupled_authored_publication {
            let event = Arc::get_mut(event).ok_or("decoded coupled publication already aliased")?;
            event._decoded_memory = memory.clone();
            match &mut event.sources {
                CanonicalCoupledAuthoredSources::Edit { .. } => {}
                CanonicalCoupledAuthoredSources::Retirement { prefix: source, .. } => {
                    let source =
                        Arc::get_mut(source).ok_or("decoded coupled retirement already aliased")?;
                    source._decoded_memory = memory.clone();
                    attach_origins(&mut source.publications, &memory)?;
                }
                CanonicalCoupledAuthoredSources::Restoration { prefix: source, .. } => {
                    let source = Arc::get_mut(source)
                        .ok_or("decoded coupled restoration already aliased")?;
                    source._decoded_memory = memory.clone();
                    attach_origins(&mut source.publications, &memory)?;
                }
            }
            for participant in &mut event.participants {
                Arc::get_mut(&mut participant.origin)
                    .ok_or("decoded coupled origin already aliased")?
                    .retain_cascade_memory(memory.clone());
                if let Some(retirement) = &mut participant.retirement {
                    let retirement = Arc::get_mut(retirement)
                        .ok_or("decoded coupled producer already aliased")?;
                    retirement._decoded_memory = memory.clone();
                    attach_origins(&mut retirement.publications, &memory)?;
                }
            }
        }
        if let CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) =
            &mut self.authority_kind
        {
            Arc::get_mut(selection)
                .ok_or("decoded graph selection already aliased")?
                ._decoded_memory = self.readmission_memory.selection.clone();
        }
        if let CanonicalCommitAuthorityKind::RetainedEntitySelection(selection) =
            &mut self.authority_kind
        {
            Arc::get_mut(selection)
                .ok_or("decoded selection already aliased")?
                ._decoded_memory = self.readmission_memory.selection.clone();
        }
        Ok(())
    }
}
fn attach_origins(
    origins: &mut [Arc<super::super::CanonicalRequiredOutputOrigin>],
    memory: &Option<Arc<worth_execution::ExecutionMemoryReservation>>,
) -> Result<(), &'static str> {
    for origin in origins {
        Arc::get_mut(origin)
            .ok_or("decoded cascade publication already aliased")?
            .retain_cascade_memory(memory.clone());
    }
    Ok(())
}
