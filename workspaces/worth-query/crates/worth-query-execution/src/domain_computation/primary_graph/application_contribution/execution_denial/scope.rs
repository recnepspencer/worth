//! Preserve a measured scope's domain error or its original execution category.
use crate::domain_computation::primary_graph::vocabulary::application_contribution::publication::publication_execution_kind;
use {crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control, crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind};
use worth_execution::{MapKernelFailure, ScopeStop, WorkCeilingDenial};

