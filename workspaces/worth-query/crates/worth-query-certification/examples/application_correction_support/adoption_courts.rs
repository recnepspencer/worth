//! Retained-source correction under an actually adopted current program.
use super::{authentication::*, edit::Value, host, program::*, query::Row, schema::NumericSchema};
use worth_query_host::facade::{
    application_entry::*, application_installation::WorthQueryProgramOwner,
    declaration::application_program::*, primary_graph::*,
};

pub fn run() {
    reverse_under_successor();
    changed_inverse_refuses();
    changed_source_meaning_refuses();
    adoption_invalidates_prepared_correction();
}
fn history_limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}
pub(super) fn adopt<P: ApplicationProgramDefinition<NumericSchema>>(
    host: &host::Host,
) -> WorthQueryPerformedBranchAdoption {
    let target = *host.supported_program::<P>().unwrap().owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host.runtime().request(&principal, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let performed = match prepared.publish(host.runtime(), &principal, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => {
            assert_eq!(performed.target(), &target);
            performed
        }
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, .. } => {
            panic!("adoption authority: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, .. } => {
            panic!("adoption admission: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("adoption execution: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(cause) => {
            panic!("adoption no effect: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(custody) => {
            panic!("adoption unpublished: {custody:?}")
        }
    };
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        &target
    );
    performed
}
fn reverse_under_successor() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source-before-adoption");
    host::edit(&host, Value::Secondary(19), "disjoint-before-adoption");
    let adopted_target = adopt::<Successor>(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let successor = *host
        .supported_program::<Successor>()
        .unwrap()
        .owned_revision();
    assert_ne!(&successor, host.owned_revision());
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-after-adoption")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("equivalent installed successor must prepare the historical inverse");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("correction under the successor must publish: {outcome:?}");
    };
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    assert_eq!(
        request.programs().inspect().unwrap().revision(),
        &successor,
        "correction keeps the actually adopted program"
    );
    assert_eq!(
        receipt.committed_product_publication().signal_posture(),
        worth_query_host::facade::product::CompositeComponentChangePosture::RetainExact
    );
    assert_eq!(
        receipt.committed_product_publication().signal_publication(),
        adopted_target.signal_publication()
    );
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        source.commit_reference()
    );
    let replay = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-after-adoption")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = replay
    else {
        panic!("retry must recover the same successor occurrence");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    println!("correction under an adopted equivalent program and exact replay passed");
}
fn changed_inverse_refuses() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source-before-changed-inverse");
    adopt::<ChangedInverse>(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    assert!(matches!(
        request.corrections().reverse(&selected).idempotency("changed-inverse").limits(history_limits()).prepare(),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::Action(
                worth_query_installation::facade::WorthQueryActionResolutionDenial::MeaningMismatch))),
    ), "matching inverse binding type cannot conceal changed installed delivery meaning");
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    println!("changed installed inverse meaning refused before effects");
}
fn adoption_invalidates_prepared_correction() {
    // The source and target can each be valid while the retained target is stale.
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source-before-preparation");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("prepared-before-adoption")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("the source program must prepare its inverse");
    };
    adopt::<Successor>(&host);
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(&outcome, WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(denial))
        if denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale),
        "adoption cannot silently rebase a prepared correction: {outcome:?}"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    println!("adoption after preparation preserves the stale-target refusal");
}

fn changed_source_meaning_refuses() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source-before-meaning-change");
    adopt::<ChangedSource>(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let result = request
        .corrections()
        .reverse(&selected)
        .idempotency("changed-source")
        .limits(history_limits())
        .prepare();
    match result {
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::ProgramMeaningChanged(requirements),
        )) => {
            assert!(!requirements.semantically_equivalent());
            assert_eq!(requirements.source(), host.owned_revision());
            assert_eq!(
                requirements.target(),
                host.supported_program::<ChangedSource>()
                    .unwrap()
                    .owned_revision()
            );
        }
        Err(other) => {
            panic!("changed source meaning must retain typed support requirements: {other:?}")
        }
        Ok(_) => panic!("changed source meaning cannot prepare or recover a correction"),
    }
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    println!("changed source meaning preserves installed comparison requirements");
}
