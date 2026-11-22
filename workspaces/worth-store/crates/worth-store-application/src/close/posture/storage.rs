//! Prepaid close-custody backing admitted with the Home so close and every
//! pending poll reuse the same funded allocation without heap churn.
use super::{ApplicationHomeRecoveryCustody, ApplicationHomeRecoveryCustodyBody};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

pub(super) struct CloseCustodySlot {
    pub(super) body: Option<ApplicationHomeRecoveryCustodyBody>,
    pub(super) memory: Option<ExecutionMemoryReservation>,
}

pub(crate) struct PreparedHomeCloseCustody {
    backing: Option<Box<CloseCustodySlot>>,
}

impl PreparedHomeCloseCustody {
    pub(crate) fn prepare(execution: &ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        let memory =
            execution.reserve_retained_memory(std::mem::size_of::<CloseCustodySlot>() as u64)?;
        Ok(Self {
            backing: Some(Box::new(CloseCustodySlot { body: None, memory })),
        })
    }

    pub(super) fn install(
        mut self,
        body: ApplicationHomeRecoveryCustodyBody,
    ) -> ApplicationHomeRecoveryCustody {
        let mut slot = self
            .backing
            .take()
            .expect("same prepaid Home close custody backing");
        assert!(
            slot.body.is_none(),
            "prepaid close custody slot is populated once"
        );
        slot.body = Some(body);
        ApplicationHomeRecoveryCustody {
            backing: Some(slot),
        }
    }
}

pub(super) fn release_slot(mut slot: Box<CloseCustodySlot>) {
    let memory = slot.memory.take();
    // Free the prepaid box before refunding its retained memory reservation.
    drop(slot);
    drop(memory);
}

impl Drop for PreparedHomeCloseCustody {
    fn drop(&mut self) {
        if let Some(slot) = self.backing.take() {
            release_slot(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{
        CancellationSource, LeaseDenial, MapKernelStop, MemoryLimitDenial, MemoryLimitLevel,
        SerialMemoryBudget, SerialRequest,
    };

    fn controller() -> u64 {
        let budget = SerialMemoryBudget::new(Some(256 << 10));
        let host =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let mut observed = None;
        ExecutionRequest::serial(&host)
            .run_native_with_checkpoint(|_| {
                observed = Some(budget.charged_memory_bytes());
            })
            .unwrap();
        assert_eq!(budget.charged_memory_bytes(), 0);
        observed.unwrap()
    }

    #[test]
    fn prepaid_close_custody_refuses_one_byte_short_and_refunds_on_drop() {
        let bill = std::mem::size_of::<CloseCustodySlot>() as u64;
        let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
        let cancellation = CancellationSource::new();
        let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&host);
        let failure = match PreparedHomeCloseCustody::prepare(&execution) {
            Err(cause) => cause,
            Ok(_) => panic!("one-short budget admitted close custody slot"),
        };
        assert!(
            matches!(
                failure,
                WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                    MemoryLimitDenial::Capacity {
                        requested,
                        admitted,
                        level: MemoryLimitLevel::Policy { ancestor: 0 },
                    }
                )) if requested == bill && admitted == bill - 1
            ),
            "{failure:?}"
        );
        assert_eq!(budget.charged_memory_bytes(), 0);
        cancellation.cancel();
        assert!(matches!(
            PreparedHomeCloseCustody::prepare(&execution),
            Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        ));

        let full = SerialMemoryBudget::new(Some(controller() + bill));
        let host = SerialRequest::from_memory(full.clone(), CancellationSource::new().token(), None);
        let execution = ExecutionRequest::serial(&host);
        let prepared = PreparedHomeCloseCustody::prepare(&execution).unwrap();
        assert_eq!(full.charged_memory_bytes(), bill);
        drop(prepared);
        assert_eq!(full.charged_memory_bytes(), 0);
    }
}
