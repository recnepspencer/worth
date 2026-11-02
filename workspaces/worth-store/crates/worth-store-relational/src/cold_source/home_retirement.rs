//! Retire this original source share after all Native-issued callbacks finish.
use super::*;
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    AnchoredRecordReadRetirement, AnchoredRecordReadRetirementDenial as Denial,
    AnchoredRecordReadRetirementProgress as Progress, RetiredAnchoredRecordReadMembership,
};
pub(crate) enum StoreColdSourceClose {
    LocalShareReleased,
    Original {
        source: StoreColdSource,
        read: Option<AnchoredRecordReadRetirement>,
        retired: Option<RetiredAnchoredRecordReadMembership>,
        complete: bool,
    },
}
pub(crate) struct RetiredStoreColdSource {
    _original: StoreColdSourceClose,
}
impl StoreColdSource {
    pub(crate) fn into_home_retirement(
        self: Arc<Self>,
        actual: &Arc<ServingPhysicalRuntime>,
    ) -> Result<StoreColdSourceClose, (Arc<Self>, Denial)> {
        if !self.belongs_to(actual) {
            return Err((self, Denial::ForeignServing));
        }
        Ok(match Arc::into_inner(self) {
            None => StoreColdSourceClose::LocalShareReleased,
            Some(source) => StoreColdSourceClose::Original {
                source,
                read: None,
                retired: None,
                complete: false,
            },
        })
    }
}
impl StoreColdSourceClose {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        actual: &Arc<ServingPhysicalRuntime>,
    ) -> Result<Progress, Denial> {
        match self {
            Self::LocalShareReleased => Ok(Progress::Complete),
            Self::Original {
                source,
                read,
                retired,
                complete,
            } => {
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
        if matches!(
            &self,
            Self::LocalShareReleased | Self::Original { complete: true, .. }
        ) {
            Ok(RetiredStoreColdSource { _original: self })
        } else {
            Err(self)
        }
    }
}
