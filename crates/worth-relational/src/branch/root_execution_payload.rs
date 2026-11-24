//! A branch root retains the original runtime-issued executable allocation.
use std::any::Any;

pub(crate) trait RootExecutionPayload: Any + Send + Sync + std::fmt::Debug {}

impl<T: Any + Send + Sync + std::fmt::Debug> RootExecutionPayload for T {}
