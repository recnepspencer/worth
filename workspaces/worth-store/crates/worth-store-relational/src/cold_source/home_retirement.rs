//! Retire this original source share after all Native-issued callbacks finish.
use super::*;
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    AnchoredRecordReadRetirement, AnchoredRecordReadRetirementDenial as Denial,
    AnchoredRecordReadRetirementProgress as Progress, RetiredAnchoredRecordReadMembership,
};
pub(crate) struct StoreColdSourceClose {
    backing: Box<ColdSourceCloseBacking>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
struct ColdSourceCloseBacking {
    original: Option<OriginalColdSourceClose>,
}
struct OriginalColdSourceClose {
    source: StoreColdSource,
    read: Option<AnchoredRecordReadRetirement>,
    retired: Option<RetiredAnchoredRecordReadMembership>,
    complete: bool,
}
pub(crate) struct RetiredStoreColdSource {
    _original: StoreColdSourceClose,
}
impl StoreColdSourceClose {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        let memory =
            request.reserve_retained_memory(std::mem::size_of::<ColdSourceCloseBacking>() as u64)?;
        Ok(Self {
            backing: Box::new(ColdSourceCloseBacking { original: None }),
            _memory: memory,
        })
    }
}
impl StoreColdSource {
    pub(crate) fn into_home_retirement(
        self: Arc<Self>,
        actual: &Arc<ServingPhysicalRuntime>,
        mut preparation: StoreColdSourceClose,
    ) -> Result<StoreColdSourceClose, (Arc<Self>, StoreColdSourceClose, Denial)> {
        if !self.belongs_to(actual) {
            return Err((self, preparation, Denial::ForeignServing));
        }
        if let Some(source) = Arc::into_inner(self) {
            preparation.backing.original = Some(OriginalColdSourceClose {
                source,
                read: None,
                retired: None,
                complete: false,
            });
        }
        Ok(preparation)
    }
}
impl StoreColdSourceClose {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        actual: &Arc<ServingPhysicalRuntime>,
    ) -> Result<Progress, Denial> {
        match &mut self.backing.original {
            None => Ok(Progress::Complete),
            Some(OriginalColdSourceClose {
                source,
                read,
                retired,
                complete,
            }) => {
                if *complete {
                    return Ok(Progress::Complete);
                }
                if !source.belongs_to(actual) {
                    return Err(Denial::ForeignServing);
                }
                request.consult().map_err(|cause| Denial::Owner(worth_store::physical_runtime::PhysicalRecordReadRetirementDenial::Execution(cause)))?;
                if read.is_none() {
                    // Exclusive original source ownership, no lock guard and
                    // no remaining Native callbacks use its former Arc address.
                    let members = source
                        .members
                        .get_mut()
                        .unwrap_or_else(|error| error.into_inner());
                    *read = members
                        .read
                        .take()
                        .map(|original| original.into_retirement());
                    return if read.is_some() {
                        Ok(Progress::Advanced)
                    } else {
                        source.serving.take();
                        *complete = true;
                        Ok(Progress::Complete)
                    };
                }
                let records = actual
                    .anchored_records()
                    .map_err(|_| Denial::ForeignServing)?;
                if read
                    .as_mut()
                    .expect("same installed selected reader")
                    .advance(&records, request)?
                    == Progress::Complete
                {
                    *retired = Some(
                        read.take()
                            .expect("completed original read")
                            .into_retired()
                            .unwrap_or_else(|_| unreachable!("actual owner reported Complete")),
                    );
                    source.serving.take();
                    *complete = true;
                    return Ok(Progress::Complete);
                }
                Ok(Progress::Advanced)
            }
        }
    }
    pub(crate) fn into_retired(self) -> Result<RetiredStoreColdSource, Self> {
        if self
            .backing
            .original
            .as_ref()
            .is_none_or(|original| original.complete)
        {
            Ok(RetiredStoreColdSource { _original: self })
        } else {
            Err(self)
        }
    }
}
