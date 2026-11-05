//! Publication request failures use the same SDK-to-Query resource vocabulary.
use {crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control, crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind};
use {
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial as Checkpoint,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption as Interruption,
};
use {
    crate::domain_computation::request_execution::execution_stop::checkpoint_denial,
    crate::domain_computation::request_execution::execution_stop::lease_denial,
    crate::domain_computation::request_execution::execution_stop::memory_denial,
};
