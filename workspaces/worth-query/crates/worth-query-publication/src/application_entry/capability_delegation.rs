use worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;

/// Why retaining or advancing a capability delegation command refused. `ProgramMismatch` means the
/// program runtime is not this request's runtime; `IdempotencyIntentDrift` means the key
/// was already recorded with a different intent.
#[derive(Debug)]
pub enum WorthQueryApplicationCapabilityDelegationDenial<PreparationDenial> {
    /// The request was refused before its first principal or projection read.
    ExecutionRequest(WorthQueryAdvancementDenial),
    Program(worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitDenial),
    ProgramMismatch,
    /// The fresh actor differs from the original full native principal.
    PrincipalMismatch,
    InputCapture(worth_query_execution::publication_boundary::operation_input_capture::WorthQueryOperationInputCaptureDenial),
    Recovery(worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial),
    PrincipalBindingInstallation(
        worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenial,
    ),
    CapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
    OperationInstallation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
    /// The idempotency key, the input or the principal identity could not be
    /// canonically encoded; the payload names the binding whose value was rejected.
    IdentityEncoding(ApplicationValueEncodeDenial),
    Authorization(
        worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenial,
    ),
    Idempotency(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    IdempotencyIntentDrift,
    Preparation(PreparationDenial),
}
