//! Accepted metadata stays inert while actual terminal receipts consume retirement.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};

impl WorthQueryInboundCustody {
    pub(in crate::domain_computation) fn terminal_sources_for_home_retirement(
        &self,
    ) -> impl Iterator<Item = &WorthQueryInboundTerminalOwnerResult> {
        self.by_message
            .values()
            .filter_map(|entry| entry.terminal.as_deref())
            .chain(self.retired_message_terminals.iter().map(Arc::as_ref))
    }
    pub(in crate::domain_computation) fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        for entry in self.by_message.values_mut() {
            request.consult().map_err(Denial::Execution)?;
            match entry.publication {
                PublicationState::Publishing | PublicationState::Recovering => {
                    return Err(Denial::Busy(Owner::PublicationReceipts))
                }
                PublicationState::Retryable
                | PublicationState::Unpublished
                | PublicationState::Terminal => {}
            }
            if let Some(terminal) = &mut entry.terminal {
                if !terminal.settle_fresh_delivery() {
                    return Err(Denial::Busy(Owner::PublicationReceipts));
                }
                let receipt = Arc::get_mut(terminal)
                    .ok_or(Denial::Busy(Owner::PublicationReceipts))?
                    .publication_mut();
                drop(receipt.take_history());
                receipt.preflight_home_retirement(request)?;
            }
        }
        for terminal in &mut self.retired_message_terminals {
            request.consult().map_err(Denial::Execution)?;
            let receipt = Arc::get_mut(terminal)
                .ok_or(Denial::Busy(Owner::PublicationReceipts))?
                .publication_mut();
            drop(receipt.take_history());
            receipt.preflight_home_retirement(request)?;
        }
        Ok(())
    }

    pub(in crate::domain_computation) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        // A completed receipt remains owned in this roster until Home reclaim.
        // The cursor is represented by the original entries moving into it.
        if let Some(mut slot) = self.by_message.first_entry() {
            if let Some(terminal) = &mut slot.get_mut().terminal {
                let progress = Arc::get_mut(terminal)
                    .ok_or(Denial::Busy(Owner::PublicationReceipts))?
                    .publication_mut()
                    .advance_home_retirement(request, native)?;
                if progress == Progress::Advanced {
                    return Ok(Progress::Advanced);
                }
            }
            let mut entry = slot.remove();
            if let Some(terminal) = entry.terminal.take() {
                self.retired_terminal_receipts.push(terminal);
            }
            self.retired_entries.push(entry);
            return Ok(Progress::Advanced);
        }
        if let Some(terminal) = self.retired_message_terminals.last_mut() {
            let progress = Arc::get_mut(terminal)
                .ok_or(Denial::Busy(Owner::PublicationReceipts))?
                .publication_mut()
                .advance_home_retirement(request, native)?;
            if progress == Progress::Complete {
                self.retired_terminal_receipts.push(
                    self.retired_message_terminals
                        .pop()
                        .expect("selected terminal"),
                );
            }
            return Ok(Progress::Advanced);
        }
        Ok(Progress::Complete)
    }
}
