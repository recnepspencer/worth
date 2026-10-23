//! Correction disclosure and derivation retain the original owner's refusal meaning.
use crate::http::server::{advancement_denial, mutation_application, query_denial};
use crate::{BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::WorkCeilingDenial,
};

pub(super) fn correction_denial(cause: WorthQueryApplicationCorrectionDenial) -> BankHttpDenial {
    use WorthQueryApplicationCorrectionDenial as D;
    match cause {
        D::ExecutionRequest(cause) => advancement_denial::advancement(cause),
        D::ProductSelection(cause) => query(
            WorthQueryApplicationRequestQueryDenial::ProductSelection(cause),
        ),
        D::Source(cause) => selection(cause),
        D::Derivation(cause) => derivation(cause),
        D::Mutation(WorthQueryApplicationRequestMutationDenial::InputCapture(cause)) => {
            input_capture(cause)
        }
        D::Mutation(cause) => mutation_application::request_mutation_denial(cause.kind()),
        D::SourceOperation(WorthQueryOperationConstructionDenial::OutputSource(cause)) => query(
            WorthQueryApplicationRequestQueryDenial::OutputSettlement(cause),
        ),
        D::SourceOperation(_) => unavailable(),
        D::SourceBindingUnavailable
        | D::RecordedOutcomeUnavailable
        | D::ProgramOwner(_)
        | D::RequiredOutputs(_) => unavailable(),
    }
}

fn query(cause: WorthQueryApplicationRequestQueryDenial) -> BankHttpDenial {
    query_denial::query_denial(bank_server::BankApplicationQueryDenial::from_request_query(
        cause,
    ))
}

fn selection(cause: WorthQueryCorrectionSelectionDenial) -> BankHttpDenial {
    use WorthQueryCorrectionSelectionDenial as D;
    match cause {
        D::Disclosure(cause) => derivation(cause),
        D::Ancestry(cause) => ancestry(cause),
        D::SourceUnavailable
        | D::SourceNotCorrectable
        | D::SourcePublicationMismatch
        | D::ForeignRuntime => BankHttpDenial::new(Kind::PermissionDenied, Next::None),
        D::ForeignPhase => BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
    }
}

fn derivation(cause: WorthQueryCorrectionDerivationDenial) -> BankHttpDenial {
    use WorthQueryApplicationRequestQueryDenial as Q;
    use WorthQueryCorrectionDerivationDenial as D;
    match cause {
        D::Principal(cause) => query(Q::PrincipalResolution(cause)),
        D::Scope(cause) => query(Q::ScopeResolution(cause)),
        D::Admission(cause) => query(Q::Admission(cause)),
        D::Read(cause) => query(Q::Execution(cause)),
        D::Product(cause) => query(Q::ProductSelection(cause)),
        D::DependencyComparison(cause) => query(Q::OutputSettlement(cause)),
        D::InputCapture(cause) => input_capture(cause),
        D::Ancestry(cause) => ancestry(cause),
        D::SourceOutput(WorthQueryCorrectionOutputProjectionDenial::Stopped(stop)) => {
            advancement_denial::request(WorkCeilingDenial::Stopped(stop))
        }
        D::TargetProgram(WorthQuerySelectedProgramInspectionDenial::ReadExecution(stop)) => {
            mutation_application::pending_execution_stop(stop)
        }
        D::Workflow(cause) => match cause.kind() {
            WorthQueryApplicationAttemptDenialKind::Cancelled => cancelled(),
            WorthQueryApplicationAttemptDenialKind::DeadlineExceeded => deadline(),
            WorthQueryApplicationAttemptDenialKind::ExecutionRequest(cause) => {
                mutation_application::pending_execution_denial(cause)
            }
            _ => stale(),
        },
        D::SubjectNotDisclosed | D::ForeignSelection => {
            BankHttpDenial::new(Kind::PermissionDenied, Next::None)
        }
        D::ForeignPhase => BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        D::TargetIncarnationChanged
        | D::OriginalSubjectChanged
        | D::SourceDiverged
        | D::ProgramMeaningChanged(_)
        | D::Domain(_) => stale(),
        D::UnsupportedSourceMeaning
        | D::MissingPreImage
        | D::SourceOutput(_)
        | D::SemanticMaterialUnavailable
        | D::MissingTargetParent
        | D::CommandKey(_)
        | D::UnsupportedTargetProgram
        | D::ProgramComparison(_)
        | D::InverseAction(_)
        | D::RequiredOutputReconciliation
        | D::TargetProgram(_)
        | D::Basis(_) => unavailable(),
    }
}

fn input_capture(cause: WorthQueryOperationInputCaptureDenial) -> BankHttpDenial {
    crate::http::server::construction_denial::capture(&cause)
}

fn ancestry(
    cause: RuntimeWorldServiceDenial<ProductBranchAncestorSelectionDenial>,
) -> BankHttpDenial {
    use CompositeHistoryCatalogDenial as Catalog;
    use ProductBranchAncestorSelectionDenial as D;
    use RuntimeWorldBranchAdmissionDenial as Branch;
    match cause {
        RuntimeWorldServiceDenial::OwnerUnavailable(_) => unavailable(),
        RuntimeWorldServiceDenial::Denied(D::Branch(Branch::CancelledBeforeEffect))
        | RuntimeWorldServiceDenial::Denied(D::History(
            CompositeAncestorSelectionDenial::Catalog(Catalog::Cancelled),
        )) => cancelled(),
        RuntimeWorldServiceDenial::Denied(D::Branch(Branch::DeadlineBeforeEffect))
        | RuntimeWorldServiceDenial::Denied(D::History(
            CompositeAncestorSelectionDenial::Catalog(Catalog::DeadlineElapsed),
        )) => deadline(),
        RuntimeWorldServiceDenial::Denied(D::History(
            CompositeAncestorSelectionDenial::SelectionLimitReached,
        )) => BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        RuntimeWorldServiceDenial::Denied(D::Branch(Branch::OwnerUnavailable)) => unavailable(),
        RuntimeWorldServiceDenial::Denied(D::History(
            CompositeAncestorSelectionDenial::Catalog(_),
        )) => unavailable(),
        RuntimeWorldServiceDenial::Denied(_) => stale(),
    }
}

fn cancelled() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Cancelled, Next::Retry)
}
fn deadline() -> BankHttpDenial {
    BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
}
fn stale() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Stale, Next::Refresh)
}
fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Unavailable, Next::Retry)
}

#[cfg(test)]
mod tests;
