mod extent;
mod inline;
mod observed;

pub(super) use extent::observe_extent;
pub(super) use inline::{observe_inline, selected_inline_target};
