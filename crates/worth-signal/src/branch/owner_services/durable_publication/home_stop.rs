//! Hold the same publication table through the actual owner admission cut.
use super::*;
use crate::durability::SignalAuthoredHomeStopDenial as Dn;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
pub(crate) struct SignalAuthoredStopPublicationGate<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    _table: std::sync::MutexGuard<'a, TaskTable<D, I, T>>,
    _unsettled: std::sync::MutexGuard<'a, Option<SignalBranchAdvanceIdentity>>,
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn prepare_home_stop(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalAuthoredStopPublicationGate<'_, D, I, T>, Dn> {
        let map = |cause| Dn::Publications(Box::new(cause));
        execution
            .consult()
            .map_err(|cause| map(SignalDurableAdvanceDenial::Request(cause)))?;
        let table = self.tasks.try_lock().map_err(|cause| {
            Dn::Owner(match cause {
                std::sync::TryLockError::WouldBlock => crate::branch::SignalOwnerStopDenial::Busy,
                std::sync::TryLockError::Poisoned(_) => {
                    crate::branch::SignalOwnerStopDenial::Poisoned
                }
            })
        })?;
        let unsettled = self.unsettled.try_lock().map_err(|cause| {
            Dn::Owner(match cause {
                std::sync::TryLockError::WouldBlock => crate::branch::SignalOwnerStopDenial::Busy,
                std::sync::TryLockError::Poisoned(_) => {
                    crate::branch::SignalOwnerStopDenial::Poisoned
                }
            })
        })?;
        if table.capture_active {
            return Err(map(
                SignalDurableAdvanceDenial::CheckpointCaptureInProgress {
                    owner: self.binding().owner(),
                },
            ));
        }
        if let Some(original) = *unsettled {
            return Err(map(SignalDurableAdvanceDenial::PublicationUnsettled {
                original,
            }));
        }
        if !table.tasks.is_empty() {
            let mut originals =
                ExecutionRequestArrayBuilder::allocate(*execution, table.tasks.len())
                    .map_err(|cause| map(SignalDurableAdvanceDenial::Backing(cause)))?;
            for original in table.tasks.keys() {
                originals
                    .push(*original)
                    .map_err(|cause| map(SignalDurableAdvanceDenial::Backing(cause)))?;
            }
            return Err(Dn::PendingOriginals {
                originals: originals
                    .seal()
                    .map_err(|cause| map(SignalDurableAdvanceDenial::Backing(cause)))?,
            });
        }
        Ok(SignalAuthoredStopPublicationGate {
            _table: table,
            _unsettled: unsettled,
        })
    }
}
