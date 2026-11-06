//! Receiver-scoped one-shot corruption of an actual duplicate comparison.
use super::{RelationalContentDenial as Denial, RelationalContentFormat as Format};
use std::cell::RefCell;
use worth_foundational::facade::CanonicalDigestId;

thread_local! {
    static ARMED: RefCell<Option<Format>> = const { RefCell::new(None) };
}
struct Scope(Option<Format>);
impl Drop for Scope {
    fn drop(&mut self) {
        ARMED.with(|armed| armed.replace(self.0.take()));
    }
}
impl crate::schema::data::RelationalSchemaRegistry {
    /// Corrupt one genuine duplicate comparison of this installed receiver.
    /// No byte override or forged identity leaves the Native owner.
    pub fn with_injected_duplicate_content_corruption_for_test<R>(
        &self,
        identity: CanonicalDigestId,
        run: impl FnOnce() -> R,
    ) -> Result<R, Denial> {
        let format = self.content_formats.get(identity)?.clone();
        let _scope = Scope(ARMED.with(|armed| armed.replace(Some(format))));
        Ok(run())
    }
}
pub(super) fn consume(format: &Format) -> bool {
    ARMED.with(|armed| {
        let mut armed = armed.borrow_mut();
        if armed
            .as_ref()
            .is_some_and(|owner| owner.same_installation(format))
        {
            armed.take();
            true
        } else {
            false
        }
    })
}
