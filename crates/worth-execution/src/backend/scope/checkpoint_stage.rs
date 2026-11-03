//! Closed original-request fragment charging through the common scope lifecycle.
use super::admission::{ScopeRequest, ScopeStorage};
use super::body::{HeldPayload, KernelVerdict, ScopeBody};
use super::execution::run_concrete_scope;
use super::ScopeStop;
use crate::backend::KernelFailure;
use crate::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use std::mem::size_of;

pub(crate) fn run_checkpoint_stage<E>(request: ExecutionRequest<'_, '_>, work: u64) -> Result<(), E>
where
    E: From<WorkCeilingDenial> + From<MapKernelStop>,
{
    let mut body = FragmentBody {
        request,
        work,
        first_stop: None,
        answer: None,
    };
    let (verdict, _) = request.with_scope_backing(|lease, serial| {
        run_concrete_scope(
            ScopeRequest {
                lease,
                serial,
                retained_bytes: 0,
                max_result_bytes: None,
                charge_parent: true,
                work_ceiling: None,
                hold: None,
                storage: ScopeStorage::Native,
                result_slot_bytes: 0,
                error_slot_bytes: 0,
            },
            &mut body,
        )
    });
    // The serializer records the actual work-checkpoint stop before any
    // later zero checkpoint or scope handoff can wrap it.
    if let Some(stop) = body.first_stop {
        return Err(E::from(stop));
    }
    match verdict {
        Ok(HeldPayload) => body.answer.expect("completed fragment ran its operation"),
        Err(ScopeStop::Failure(KernelFailure::Stop(MapKernelStop::NestedStopped))) => {
            match body.answer {
                Some(Err(inner)) => Err(inner),
                _ => Err(E::from(MapKernelStop::NestedStopped)),
            }
        }
        Err(stop) => Err(E::from(denial(stop))),
    }
}

struct FragmentBody<'request, 'authority, E> {
    request: ExecutionRequest<'request, 'authority>,
    work: u64,
    first_stop: Option<MapKernelStop>,
    answer: Option<Result<(), E>>,
}

impl<E: From<WorkCeilingDenial> + From<MapKernelStop>> ScopeBody for FragmentBody<'_, '_, E> {
    fn invoke(
        &mut self,
        context: &mut MapKernelContext<'_, '_>,
        _: Option<crate::ExecutionResultMemory<'_>>,
    ) -> KernelVerdict {
        self.answer = Some(if context.has_accounting() {
            self.request.consult().map_err(E::from).and_then(|()| {
                self.request
                    .in_scope(|lease| {
                        let mut memory = ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                            .map_err(|stop| E::from(WorkCeilingDenial::Admission(stop)))?;
                        self.request.with_scope_backing(|lease, serial| {
                            let mut leaf = FragmentLeaf {
                                work: self.work,
                                first_stop: &mut self.first_stop,
                            };
                            let (result, _) = run_concrete_scope(
                                ScopeRequest {
                                    lease,
                                    serial,
                                    retained_bytes: 0,
                                    max_result_bytes: None,
                                    charge_parent: true,
                                    work_ceiling: self.request.work_ceiling(),
                                    hold: Some(&mut memory),
                                    storage: ScopeStorage::Unit,
                                    // The original measured unit owns its zero
                                    // heap-size field in addition to unit.
                                    result_slot_bytes: size_of::<u64>(),
                                    error_slot_bytes: 0,
                                },
                                &mut leaf,
                            );
                            result
                                .map(|HeldPayload| ())
                                .map_err(|stop| E::from(denial(stop)))
                        })
                    })
                    .map_err(E::from)?
            })
        } else {
            self.first_stop = context.poll().err();
            Ok(())
        });
        context.poll()?;
        Ok(HeldPayload)
    }

    fn payload_bytes(&self) -> u64 {
        0
    }
    fn kept_bytes(&self) -> Option<u64> {
        Some(0)
    }
    fn discard(&mut self) {}
}

struct FragmentLeaf<'a> {
    work: u64,
    first_stop: &'a mut Option<MapKernelStop>,
}
impl ScopeBody for FragmentLeaf<'_> {
    fn invoke(
        &mut self,
        context: &mut MapKernelContext<'_, '_>,
        _: Option<crate::ExecutionResultMemory<'_>>,
    ) -> KernelVerdict {
        context.checkpoint(0)?;
        *self.first_stop = context.checkpoint(self.work).err();
        // The opaque callback returned unit even when its work checkpoint
        // stopped. Its trailing zero checkpoint must still execute.
        context.checkpoint(0)?;
        Ok(HeldPayload)
    }
    fn payload_bytes(&self) -> u64 {
        0
    }
    fn kept_bytes(&self) -> Option<u64> {
        Some(0)
    }
    fn discard(&mut self) {}
}

fn denial(stop: ScopeStop<HeldPayload>) -> WorkCeilingDenial {
    match stop {
        ScopeStop::Admission(cause) => WorkCeilingDenial::Admission(cause),
        ScopeStop::Failure(KernelFailure::Stop(stop)) => WorkCeilingDenial::Stopped(stop),
        ScopeStop::Failure(KernelFailure::Panic) => WorkCeilingDenial::Panicked,
        ScopeStop::Failure(KernelFailure::Domain(_) | KernelFailure::ResultCapacityExceeded) => {
            unreachable!("closed zero-result fragment emits neither domain nor capacity failure")
        }
    }
}
