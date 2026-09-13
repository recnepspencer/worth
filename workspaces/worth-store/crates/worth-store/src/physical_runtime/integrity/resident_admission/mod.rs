pub(in crate::physical_runtime) mod denial;
pub(in crate::physical_runtime) mod extent;
pub(in crate::physical_runtime) mod free_space;
pub(in crate::physical_runtime) mod load;
pub(in crate::physical_runtime) mod page;
mod record_binding;
pub(in crate::physical_runtime) mod root_manifest;
pub(in crate::physical_runtime) mod root_protocol;
pub(in crate::physical_runtime) mod root_tree;
mod source_scope;
pub(in crate::physical_runtime) use source_scope::artifact_matches_scope;

#[cfg(test)]
mod tests;
