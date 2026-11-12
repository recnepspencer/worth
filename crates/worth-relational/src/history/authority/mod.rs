mod replay_retention;
#[cfg(test)]
mod test_mutation;

use crate::runtime::RelationalRuntime;

pub struct HistoryAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub fn history_authority(&self) -> HistoryAuthority<'_> {
        HistoryAuthority::new(self)
    }
}

impl<'runtime> HistoryAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub(crate) fn runtime(&self) -> &RelationalRuntime {
        self.runtime
    }
}
