//! An owned one-shot callback at the adoption preparation boundary.
use std::sync::{Arc, Mutex};
type Callback = Box<dyn FnOnce() + Send>;
#[derive(Default)]
pub(super) struct AdoptionPreparationControl {
    callback: Mutex<Option<Callback>>,
}
impl AdoptionPreparationControl {
    pub(super) fn reached(&self) {
        let callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(callback) = callback {
            callback();
        }
    }
}
pub struct WorthQueryAdoptionPreparationForTest {
    control: Arc<AdoptionPreparationControl>,
    previous: Option<Callback>,
}
impl Drop for WorthQueryAdoptionPreparationForTest {
    fn drop(&mut self) {
        *self
            .control
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = self.previous.take();
    }
}
impl super::WorthQueryProductRuntime {
    pub fn before_adoption_preparation_for_test(
        &self,
        callback: impl FnOnce() + Send + 'static,
    ) -> WorthQueryAdoptionPreparationForTest {
        let previous = self
            .adoption_preparation
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(Box::new(callback));
        WorthQueryAdoptionPreparationForTest {
            control: Arc::clone(&self.adoption_preparation),
            previous,
        }
    }
}
