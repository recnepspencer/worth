#[cfg(test)]
mod aspect_history;
#[cfg(test)]
mod divergence;
mod event_demand;
#[cfg(test)]
mod graph;
mod records;
mod resolution;
mod selected_root_resolution;

use crate::runtime::RelationalRuntime;

pub struct LineageAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn lineage_access(&self) -> LineageAccess<'_> {
        LineageAccess::new(self)
    }
}

impl<'runtime> LineageAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
