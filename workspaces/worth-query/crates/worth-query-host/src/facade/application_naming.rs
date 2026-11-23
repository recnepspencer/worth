//! Persistent names an application holds and resolves: the declared
//! vocabulary, the tracked reader over an admitted observation, its typed
//! refusals and the dependency footprint a later commit can stale. Names are
//! admitted only from committed births; nothing here forges one.

pub use worth_query_declaration::facade::application_naming::{
    ErasedWorthQueryName, NameResolutionDenial, NameSelection, NameSelectionDenial, ResolvedName,
    WorthQueryName,
};
pub use worth_query_execution::facade::application_naming::{
    ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial, NameReadCurrentness,
    NameReadDependencies, NameReadStaleness, PersistentNameReadVeto, WorthQueryDecisionNameDenial,
};
