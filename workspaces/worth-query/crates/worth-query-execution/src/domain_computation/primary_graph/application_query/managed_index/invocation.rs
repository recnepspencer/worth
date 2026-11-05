//! Pure callbacks run outside cache locks under their actual original request.
use super::{
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexProviderFailure,
    WorthQueryManagedIndexProviderStop as ProviderStop, WorthQueryManagedIndexStop as Stop,
};
use std::cell::RefCell;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};

/// A controlled request refusal or the provider's retained concrete domain cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexWorkDenial {
    Controlled(Stop),
    Provider(WorthQueryManagedIndexProviderFailure),
}
impl worth_execution::ChargedBytes for WorthQueryManagedIndexWorkDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Provider(cause) => cause.retained_payload_bytes(),
            Self::Controlled(_) => 0,
        }
    }
}
impl std::fmt::Display for WorthQueryManagedIndexWorkDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for WorthQueryManagedIndexWorkDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provider(cause) => Some(cause),
            Self::Controlled(_) => None,
        }
    }
}

impl From<Stop> for WorthQueryManagedIndexWorkDenial {
    fn from(cause: Stop) -> Self {
        Self::Controlled(cause)
    }
}

pub fn controlled<R>(
    execution: &ExecutionRequest<'_, '_>,
    body: impl FnOnce(&Control<'_>) -> Result<R, WorthQueryManagedIndexWorkDenial>,
) -> Result<R, WorthQueryManagedIndexWorkDenial> {
    let first = RefCell::new(None);
    let mut returned = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        let accounted = context.has_accounting();
        let context = RefCell::new(context);
        let checkpoint = |work, source: Option<&worth_relational::facade::runtime::CustomInvariantReadBudget<'_>>| {
            if let Some(source) = source {
                return source.checkpoint_native(&mut context.borrow_mut(), work).map_err(|cause| {
                    use worth_relational::facade::runtime::CustomInvariantNativeCheckpointDenial as Denial;
                    match cause {
                        Denial::Source(cause) => Stop::NativeSource(cause),
                        Denial::Request(cause) => Stop::Request(WorkCeilingDenial::Stopped(cause)),
                    }
                });
            }
            let result = if accounted {
                context.borrow_mut().checkpoint(work)
            } else {
                context.borrow_mut().poll()
            };
            result.map_err(|cause| Stop::Request(WorkCeilingDenial::Stopped(cause)))
        };
        let poll = || context.borrow_mut().poll();
        let reserve = |bytes| execution.reserve_retained_memory(bytes).map_err(Stop::Request);
        let control = Control::new(&checkpoint, &poll, &first, &reserve);
        let result = body(&control);
        // A callback cannot substitute success or a domain refusal for a stop
        // that it caught from an original request checkpoint or storage door.
        returned = Some(match control.poll() {
            Err(cause) => Err(cause.into()),
            Ok(()) => result,
        });
    });
    match (outcome, returned) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Stop::Request(cause).into()),
        (Ok(_), None) => unreachable!("completed original provider scope"),
    }
}

pub fn provider_result<R, D>(
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
    result: Result<R, ProviderStop<D>>,
) -> Result<R, WorthQueryManagedIndexWorkDenial>
where
    D: Copy + Eq + std::error::Error + Send + Sync + 'static,
{
    control.poll()?;
    match result {
        Ok(value) => Ok(value),
        Err(ProviderStop::Controlled(cause)) => Err(control.refuse(cause).into()),
        Err(ProviderStop::Domain(cause)) => {
            let cause = WorthQueryManagedIndexProviderFailure::retain(execution, control, cause)?;
            Err(WorthQueryManagedIndexWorkDenial::Provider(cause))
        }
    }
}

/// The managed decoder and control share the first actual refusal, while new
/// outer SDK refusals remain authoritative rather than being replaced by a
/// saved callback result.
pub fn decoded_provider<R>(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    body: impl FnOnce(
        &Control<'_>,
        &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
    ) -> Result<R, WorthQueryManagedIndexWorkDenial>,
) -> Result<R, WorthQueryManagedIndexWorkDenial> {
    use crate::domain_computation::primary_graph::value_decode::{
        ValueDecodeState, WorthQueryValueDecodeDenial as Decode,
    };
    let decode = ValueDecodeState::admitted(bindings.clone());
    let mut returned = None;
    let mut outer_refusal = None;
    let outcome = decode.run_with_scope_refusal(
        execution,
        &mut |cause| {
            outer_refusal.get_or_insert(cause);
        },
        |session| {
            returned = Some(controlled(execution, |control| {
                let refusal = |cause| {
                    control.refuse(Stop::ValueDecode(cause));
                };
                let session = session.observing(&refusal);
                body(control, &session)
            }));
        },
    );
    if let Some(cause) = outer_refusal
        && !matches!(
            cause,
            WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)
        )
    {
        // The same request stop keeps its concrete first owner; a
        // genuinely different SDK envelope refusal remains authoritative.
        if let Some(Err(WorthQueryManagedIndexWorkDenial::Controlled(first))) = &returned {
            let same = match first {
                Stop::Request(original) | Stop::ValueDecode(Decode::Scope(original)) => {
                    *original == cause
                }
                Stop::ValueDecode(Decode::Checkpoint(original)) => {
                    cause == WorkCeilingDenial::Stopped(*original)
                }
                _ => false,
            };
            if same {
                return Err(first.clone().into());
            }
        }
        return Err(Stop::ValueDecode(Decode::Scope(cause)).into());
    }
    match (outcome, returned) {
        (Ok(()), Some(result)) => result,
        (Err(cause), saved) => {
            // Distinct real SDK envelope causes were handled above. All
            // decoder failures now preserve the shared first controlled owner,
            // including actual memory, reservation and checkpoint refusals.
            if let Some(Err(WorthQueryManagedIndexWorkDenial::Controlled(first))) = saved {
                return Err(first.into());
            }
            Err(Stop::ValueDecode(cause).into())
        }
        (Ok(()), None) => unreachable!("completed managed decode invocation"),
    }
}
