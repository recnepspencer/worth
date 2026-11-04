pub(crate) struct Plain(pub(crate) usize);

pub(crate) struct Thing;

pub(crate) fn measure() -> usize {
    1
}

impl Plain {
    pub(crate) fn twice(&self) -> usize {
        self.0 * 2
    }
}
