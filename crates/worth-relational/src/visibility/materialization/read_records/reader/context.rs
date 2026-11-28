use crate::runtime::RelationalRuntime;

pub struct VisibilityReadContext<'runtime> {
    pub(in crate::visibility::materialization::read_records::reader) runtime:
        &'runtime RelationalRuntime,
}

impl<'runtime> VisibilityReadContext<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub(crate) const fn runtime(&self) -> &'runtime RelationalRuntime {
        self.runtime
    }
}

impl RelationalRuntime {
    pub fn read_truth(&self) -> VisibilityReadContext<'_> {
        VisibilityReadContext::new(self)
    }
}
