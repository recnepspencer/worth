//! The original Native and Store source retire before physical close.
use super::RelationalStoreSection;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::{
    RelationalRuntimeCloseDenial, RelationalSourceClose, RelationalSourceCloseDenial,
    RelationalSourceCloseProgress, RetiredSourceRelationalAllocation,
};
use worth_store::physical_runtime::{AnchoredRecordReadRetirementProgress, ServingPhysicalRuntime};
mod preparation;
mod progression;
pub use preparation::PreparedRelationalSectionClose;

enum ColdSourceCustody {
    Serving {
        source: Arc<crate::cold_source::StoreColdSource>,
        preparation: crate::cold_source::StoreColdSourceClose,
    },
    Retiring(crate::cold_source::StoreColdSourceClose),
    Retired {
        _original: crate::cold_source::RetiredStoreColdSource,
    },
}
pub struct ClosingRelationalSection {
    pub(super) source: Option<super::readmission::source::ReadmittedRelationalSource>,
    runtime: Option<RelationalSourceClose>,
    retired_runtime: Option<RetiredSourceRelationalAllocation>,
    cold_source: Option<ColdSourceCustody>,
    serving: Option<Arc<ServingPhysicalRuntime>>,
    checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
    complete: bool,
}
/// Original inert Native, provider, directory and Store source payloads.
#[must_use]
pub struct RetiredRelationalSectionAllocation {
    _original: ClosingRelationalSection,
}
impl RetiredRelationalSectionAllocation {
    pub fn reclaim(self) {
        drop(self);
    }
}

#[derive(Debug)]
pub enum RelationalSectionCloseDenial {
    SharedRuntimeCustody,
    PendingCodecTransition,
    Native(RelationalRuntimeCloseDenial),
    Source(RelationalSourceCloseDenial),
    Physical(worth_store::physical_runtime::AnchoredRecordReadRetirementDenial),
    ForeignServing,
}
fn native_denial(cause: RelationalSourceCloseDenial) -> RelationalSectionCloseDenial {
    match cause {
        RelationalSourceCloseDenial::SharedRuntime => {
            RelationalSectionCloseDenial::SharedRuntimeCustody
        }
        RelationalSourceCloseDenial::Native(cause) => RelationalSectionCloseDenial::Native(cause),
        cause @ (RelationalSourceCloseDenial::Busy
        | RelationalSourceCloseDenial::Medium(_)
        | RelationalSourceCloseDenial::Member(_)
        | RelationalSourceCloseDenial::Checkpoint(_)
        | RelationalSourceCloseDenial::Execution(_)) => RelationalSectionCloseDenial::Source(cause),
    }
}
impl RelationalStoreSection {
    pub fn prepare_close(
        self,
        preparation: PreparedRelationalSectionClose,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ClosingRelationalSection,
        (
            Self,
            PreparedRelationalSectionClose,
            RelationalSectionCloseDenial,
        ),
    > {
        if matches!(&self.source, Some(super::readmission::source::ReadmittedRelationalSource::Recovered { authority, .. })
            if authority.codec_transition_pending())
        {
            return Err((
                self,
                preparation,
                RelationalSectionCloseDenial::PendingCodecTransition,
            ));
        }
        let Self {
            source,
            runtime,
            cold_source,
            serving,
            checkpoint_selection,
        } = self;
        let PreparedRelationalSectionClose { native, cold } = preparation;
        let mut runtime = match runtime.into_source_close(native, request, cold_source.as_ref()) {
            Ok(runtime) => runtime,
            Err((runtime, preparation, cause)) => {
                return Err((
                    Self {
                        source,
                        runtime,
                        cold_source,
                        serving,
                        checkpoint_selection,
                    },
                    PreparedRelationalSectionClose {
                        native: preparation,
                        cold,
                    },
                    native_denial(cause),
                ))
            }
        };
        // Finish reversible graph validation before handing away Serving.
        // A source-capable fork must refuse before any cache or source retires.
        if let Err(cause) = runtime.preflight(request, cold_source.as_ref()) {
            let (runtime, preparation) = runtime
                .resume()
                .unwrap_or_else(|_| unreachable!("preflight cannot seal a binding"));
            return Err((
                Self {
                    source,
                    runtime: Arc::new(runtime),
                    cold_source,
                    serving,
                    checkpoint_selection,
                },
                PreparedRelationalSectionClose {
                    native: preparation,
                    cold,
                },
                native_denial(cause),
            ));
        }
        Ok(ClosingRelationalSection {
            source,
            runtime: Some(runtime),
            retired_runtime: None,
            cold_source: Some(ColdSourceCustody::Serving {
                source: cold_source,
                preparation: cold,
            }),
            serving: Some(serving),
            checkpoint_selection,
            complete: false,
        })
    }
}
pub(super) fn prepare_attached_close(
    attached: super::unshared::AttachedRelationalRuntime,
    source: Option<super::readmission::source::ReadmittedRelationalSource>,
    preparation: PreparedRelationalSectionClose,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    ClosingRelationalSection,
    (
        super::unshared::AttachedRelationalRuntime,
        Option<super::readmission::source::ReadmittedRelationalSource>,
        PreparedRelationalSectionClose,
        RelationalSectionCloseDenial,
    ),
> {
    if matches!(&source, Some(super::readmission::source::ReadmittedRelationalSource::Recovered { authority, .. }) if authority.codec_transition_pending())
    {
        return Err((
            attached,
            source,
            preparation,
            RelationalSectionCloseDenial::PendingCodecTransition,
        ));
    }
    let super::unshared::AttachedRelationalRuntime {
        runtime,
        cold_source,
        serving,
        checkpoint_selection,
    } = attached;
    let PreparedRelationalSectionClose { native, cold } = preparation;
    let mut runtime = match runtime.into_owned_source_close(native, request, cold_source.as_ref()) {
        Ok(runtime) => runtime,
        Err((runtime, preparation, cause)) => {
            return Err((
                super::unshared::AttachedRelationalRuntime {
                    runtime,
                    cold_source,
                    serving,
                    checkpoint_selection,
                },
                source,
                PreparedRelationalSectionClose {
                    native: preparation,
                    cold,
                },
                native_denial(cause),
            ))
        }
    };
    if let Err(cause) = runtime.preflight(request, cold_source.as_ref()) {
        let (runtime, preparation) = runtime
            .resume()
            .unwrap_or_else(|_| unreachable!("preflight cannot seal a binding"));
        return Err((
            super::unshared::AttachedRelationalRuntime {
                runtime,
                cold_source,
                serving,
                checkpoint_selection,
            },
            source,
            PreparedRelationalSectionClose {
                native: preparation,
                cold,
            },
            native_denial(cause),
        ));
    }
    Ok(ClosingRelationalSection {
        source,
        runtime: Some(runtime),
        retired_runtime: None,
        cold_source: Some(ColdSourceCustody::Serving {
            source: cold_source,
            preparation: cold,
        }),
        serving: Some(serving),
        checkpoint_selection,
        complete: false,
    })
}

impl ClosingRelationalSection {
    pub fn resume(self) -> Result<(RelationalStoreSection, PreparedRelationalSectionClose), Self> {
        let Self {
            source,
            runtime,
            retired_runtime,
            cold_source,
            serving,
            checkpoint_selection,
            complete,
        } = self;
        let Some(runtime) = runtime else {
            return Err(Self {
                source,
                runtime,
                retired_runtime,
                cold_source,
                serving,
                checkpoint_selection,
                complete,
            });
        };
        match runtime.resume() {
            Ok((runtime, preparation)) => {
                let Some(ColdSourceCustody::Serving {
                    source: cold_source,
                    preparation: cold,
                }) = cold_source
                else {
                    unreachable!("Native pre-seal precedes Store source retirement");
                };
                Ok((
                    RelationalStoreSection {
                        source,
                        runtime: Arc::new(runtime),
                        cold_source,
                        serving: serving.expect("pre-seal retains physical custody"),
                        checkpoint_selection,
                    },
                    PreparedRelationalSectionClose {
                        native: preparation,
                        cold,
                    },
                ))
            }
            Err(runtime) => Err(Self {
                source,
                runtime: Some(runtime),
                retired_runtime,
                cold_source,
                serving,
                checkpoint_selection,
                complete,
            }),
        }
    }
    pub fn into_retired(self) -> Result<RetiredRelationalSectionAllocation, Self> {
        if !self.complete {
            return Err(self);
        }
        Ok(RetiredRelationalSectionAllocation { _original: self })
    }
}
