//! One actual task-table gate excludes publication while original source is captured.
use super::*;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
struct Capture<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    state: Arc<SignalDurablePublicationState<D, I, T>>,
}
impl<D, I, T> Drop for Capture<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn drop(&mut self) {
        self.state
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .capture_active = false;
    }
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn capture_authored_home_image(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::durability::SignalAuthoredHomeImage, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        execution.consult().map_err(Dn::Request)?;
        let mut table = self
            .tasks
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?;
        if table.capture_active {
            return Err(Dn::CheckpointCaptureInProgress {
                owner: self.binding().owner(),
            });
        }
        if let Some(original) = *self
            .unsettled
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
        {
            return Err(Dn::PublicationUnsettled { original });
        }
        if !table.tasks.is_empty() {
            let mut originals =
                ExecutionRequestArrayBuilder::allocate(*execution, table.tasks.len())
                    .map_err(Dn::Backing)?;
            for original in table.tasks.keys() {
                originals.push(*original).map_err(Dn::Backing)?;
            }
            return Err(Dn::CheckpointOriginalsPending {
                originals: originals.seal().map_err(Dn::Backing)?,
            });
        }
        let bounds = self
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .frontiers
            .clone();
        table.capture_active = true;
        let capture = Capture {
            state: Arc::clone(self),
        };
        drop(table);
        match self.medium.publication_task_posture() {
            crate::durability::SignalPublicationTaskPosture::Terminal => {}
            crate::durability::SignalPublicationTaskPosture::Busy => {
                return Err(Dn::Medium(
                    crate::durability::SignalDurableMediumDenial::PublicationBusy,
                ))
            }
            crate::durability::SignalPublicationTaskPosture::Unsettled => {
                return Err(Dn::Medium(
                    crate::durability::SignalDurableMediumDenial::PublicationUnresolved,
                ))
            }
        }
        let source = self
            .medium
            .capture_recovery_source(execution)
            .map_err(Dn::Medium)?;
        let image =
            crate::durability::SignalAuthoredHomeImage::capture(source.as_ref(), bounds, execution)
                .map_err(Dn::Readmission)?;
        drop(capture);
        Ok(image)
    }
}
