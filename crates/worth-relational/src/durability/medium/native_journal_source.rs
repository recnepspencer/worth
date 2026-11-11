//! The local journal edge can retire without revoking another source owner.
use super::{RelationalDurableColdSource, RelationalDurableMemberRetirementRefusal as Refusal};
use std::sync::Arc;
#[derive(Debug)]
pub(crate) struct NativeJournalSourceEdge(Option<Arc<dyn RelationalDurableColdSource>>);
impl NativeJournalSourceEdge {
    pub(crate) fn new(source: Arc<dyn RelationalDurableColdSource>) -> Self {
        Self(Some(source))
    }
    pub(crate) fn preflight(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Refusal> {
        if self
            .0
            .as_ref()
            .is_some_and(|original| !std::ptr::addr_eq(original.as_ref(), source))
        {
            return Err(Refusal::ForeignSource);
        }
        Ok(())
    }
    pub(crate) fn retire(
        &mut self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Refusal> {
        self.preflight(source)?;
        // The separately borrowed authentic source proves this is nonfinal.
        self.0.take();
        Ok(())
    }
}
impl std::ops::Deref for NativeJournalSourceEdge {
    type Target = Arc<dyn RelationalDurableColdSource>;
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("only serving journal edges support reads")
    }
}
