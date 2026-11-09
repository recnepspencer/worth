//! A temporary loan restores the original driver before outer custody drops.
use super::*;
use crate::lifecycle::WorldDurableCreationCustody;
pub(crate) struct ActiveDurableCreationLease {
    record: Arc<ActiveAttemptRecord>,
    original: Option<Box<WorldDurableCreationCustody>>,
}
impl ActiveAttemptCustody {
    pub(crate) fn retains_durable_creation(&self) -> bool {
        self.record
            .state()
            .resources
            .as_ref()
            .is_some_and(|r| r.creation.as_ref().is_some_and(|c| c.durable.is_some()))
    }
    pub(crate) fn retains_durable_attempt(&self) -> bool {
        self.record.state().resources.as_ref().is_some_and(|r| {
            r.durable_open.is_some() || r.creation.as_ref().is_some_and(|c| c.durable.is_some())
        })
    }
    pub(crate) fn install_durable_creation(&mut self, body: Box<WorldDurableCreationCustody>) {
        let mut lease = self.lease_resources();
        let creation = lease
            .resources_mut()
            .creation
            .as_mut()
            .expect("creation admission");
        assert!(creation.durable.is_none());
        creation.durable = Some(body);
    }
    pub(crate) fn loan_durable_creation(&mut self) -> ActiveDurableCreationLease {
        let body = self
            .lease_resources()
            .resources_mut()
            .creation
            .as_mut()
            .expect("creation admission")
            .durable
            .take()
            .expect("one driver");
        ActiveDurableCreationLease {
            record: self.record.clone(),
            original: Some(body),
        }
    }
    pub(crate) fn take_creation_response(&mut self) -> crate::branch::ProductBranchObservation {
        self.lease_resources()
            .resources_mut()
            .creation
            .as_mut()
            .expect("creation")
            .observation
            .take()
            .expect("prepared response")
    }
    pub(crate) fn creation_snapshot(&mut self) -> crate::branch::ProductBranchReferenceSnapshot {
        self.lease_resources()
            .resources_mut()
            .creation
            .as_ref()
            .expect("creation")
            .cell
            .as_ref()
            .expect("prepared cell")
            .atomic_snapshot()
    }
}
impl std::ops::Deref for ActiveDurableCreationLease {
    type Target = WorldDurableCreationCustody;
    fn deref(&self) -> &Self::Target {
        self.original.as_ref().expect("original loan")
    }
}
impl std::ops::DerefMut for ActiveDurableCreationLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.original.as_mut().expect("original loan")
    }
}
impl Drop for ActiveDurableCreationLease {
    fn drop(&mut self) {
        let body = self.original.take().expect("restore once");
        let progress = Arc::new(body.progress.retained_image());
        let mut state = self.record.state();
        let creation = state
            .resources
            .as_mut()
            .expect("resources restored before driver")
            .creation
            .as_mut()
            .expect("creation");
        assert!(creation.durable.is_none());
        let retirement = if body.is_pending() {
            creation.durable = Some(body);
            None
        } else {
            Some(body)
        };
        state.progress = progress;
        drop(state);
        drop(retirement);
    }
}
