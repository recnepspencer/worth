//! Each family's vocabulary: the refusals, observations, identities and evidence its
//! operations exchange, held below every family so one family can name another's without a
//! cycle. Each child holds one family's items, moved whole from that family.
pub(crate) mod application_aftermath;
pub(crate) mod authorization;
pub(crate) mod execution_runtime;
pub(crate) mod managed_run;
pub(crate) mod operation_binding;
pub(crate) mod primary_graph;
pub(crate) mod provider_session;
pub(crate) mod request_execution;
