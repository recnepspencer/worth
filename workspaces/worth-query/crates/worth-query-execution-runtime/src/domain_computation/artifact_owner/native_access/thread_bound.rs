use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WorthQueryArtifactThreadBound {
    _not_send_or_sync: PhantomData<Rc<()>>,
}

impl WorthQueryArtifactThreadBound {
    pub(super) const fn new() -> Self {
        Self {
            _not_send_or_sync: PhantomData,
        }
    }
}
