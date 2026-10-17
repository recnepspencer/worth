impl Span {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
use crate::{call, text};
#[derive(Clone, Copy)]
pub struct Span(pub(crate) u32, std::marker::PhantomData<std::rc::Rc<()>>);
impl Span {
    pub fn call_site() -> Self {
        Self::from_handle(call(10, 0, 0, 0))
    }
    pub fn mixed_site() -> Self {
        Self::from_handle(call(11, 0, 0, 0))
    }
    pub fn resolved_at(&self, other: Self) -> Self {
        Self::from_handle(call(12, self.0, other.0, 0))
    }
    pub fn located_at(&self, other: Self) -> Self {
        Self::from_handle(call(13, self.0, other.0, 0))
    }
    pub fn join(&self, other: Self) -> Option<Self> {
        let h = call(14, self.0, other.0, 0);
        (h != 0).then_some(Self::from_handle(h))
    }
    pub fn source_text(&self) -> Option<String> {
        let h = call(15, self.0, 0, 0);
        (h != 0).then(|| text(h))
    }
    pub fn start(&self) -> Self {
        Self::from_handle(call(16, self.0, 0, 0))
    }
    pub fn end(&self) -> Self {
        Self::from_handle(call(17, self.0, 0, 0))
    }
    pub fn line(&self) -> usize {
        call(18, self.0, 0, 0) as usize
    }
    pub fn column(&self) -> usize {
        call(19, self.0, 0, 0) as usize
    }
    pub fn file(&self) -> String {
        text(call(20, self.0, 0, 0))
    }
    pub fn local_file(&self) -> Option<std::path::PathBuf> {
        let h = call(21, self.0, 0, 0);
        (h != 0).then(|| text(h).into())
    }
}
impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&text(call(22, self.0, 0, 0)))
    }
}
