//! The region output handlers only the fixture's own programs install: one for
//! a program that runs no region output, and one whose decision keeps a
//! preserved artifact without reading its Length.

use super::region_output::RegionOutputHandler;
use super::*;

impl<Schema: TopologyModelBinding> RegionOutputHandler<Schema> {
    /// The handler of a program that runs no region output.
    pub(super) const fn idle() -> Self {
        Self {
            run: None,
            generated: false,
            preserved: false,
        }
    }

    pub(super) fn preserving_payload(mut self) -> Self {
        self.preserved = true;
        self
    }
}
