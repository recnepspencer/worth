//! A genuine non-leaf SDK graph: discovery reads the first family's output.
use super::*;
mod courts;
pub(super) mod program;

mod partition;

mod historical_scope;

mod member_children;

pub(super) use program::{
    Child, ChildInput, Input, Output, Parent, Program as HierarchyProgram, SeedInput,
};

mod recording_parent;

mod source_edits;
