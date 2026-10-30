//! Real stable registration and adoption-derived results remain separate ordinary outcomes.
use super::{
    super::{
        application::{
            candidate::{adoption::Adopted, StageEvent, Staging},
            model::Model,
            storage::{LoadedProposal, RegisteredClaim, Storage},
        },
        attempts::{self, data::*},
        authentication::{self, Actor, Caller},
        host::{self, Host},
        schema::data::AuthoredIntent,
    },
    retention_action::edit,
};
use worth_query_host::facade::{
    application_entry::*, product::*, runtime::ExecutionAllocationPolicy,
};
mod disclosure;
mod handoff;

fn adopt<'host>(
    model: &mut Model<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    attempt: u64,
) -> Adopted<'host> {
    let mut staging = Staging::prepare_frozen(model, storage, caller, loaded, attempt)
        .unwrap_or_else(|cause| panic!("prepare exact frozen edit: {cause:?}"));
    for _ in &loaded.proposal().intents {
        assert!(matches!(
            staging.advance(model, caller),
            StageEvent::StepCompleted
        ));
    }
    let candidate = staging.branch();
    let ready = staging
        .finish()
        .unwrap_or_else(|_| panic!("all ordinary steps completed"));
    let adopted = ready
        .adopt(model, caller)
        .unwrap_or_else(|failure| panic!("adopt actual slot: {:?}", failure.cause));
    assert_eq!(model.branch(), candidate);
    adopted
}
fn describe(
    host: &Host,
    caller: &Caller<'_>,
    branch: WorthQueryProductBranch,
) -> WorthQueryExpectedProductBasisDescription {
    host.with_application_advancement(caller.scope, |phase| {
        host.on_branch(branch)
            .select(&phase)
            .unwrap()
            .describe_basis()
    })
    .unwrap()
}

#[test]
fn stable_claim_binds_ordered_input_and_records_only_the_actual_adoption() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 801, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 802, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            803,
            vec![
                AuthoredIntent::Retention(edit(&model, 2)),
                AuthoredIntent::Retention(edit(&model, 3)),
            ],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.describe(&reviewer).unwrap();
    let registered: RegisteredClaim = storage.claim(&model, &reviewer, &loaded, 804).unwrap();
    assert!(matches!(
        registered.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    let claim = registered.reference().clone();
    drop(registered);
    assert_eq!(model.describe(&reviewer).unwrap(), source);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let original = loaded.input(804);
    let mut reordered = original.clone();
    reordered.proposal.intents.reverse();
    let mut removed = original.clone();
    removed.proposal.intents.pop();
    let mut empty = original.clone();
    empty.proposal.intents.clear();
    let mut value = original.clone();
    let AuthoredIntent::Retention(edit) = &mut value.proposal.intents[1];
    edit.days = 4;
    let mut target = original.clone();
    let AuthoredIntent::Retention(edit) = &mut target.proposal.intents[1];
    edit.target = other.target.clone();
    for changed in [reordered, removed, empty, value, target] {
        // Only the ordered input changed: record identity/revision/basis stay equal.
        assert_eq!(changed.reference, original.reference);
        assert_eq!(
            changed.proposal.expected_basis,
            original.proposal.expected_basis
        );
        let request = host
            .request(&reviewer.principal, reviewer.scope)
            .on_branch(storage.branch());
        let operation = storage.claim_operation(&reviewer, &changed, 804).unwrap();
        let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
            panic!("owning drift terminal")
        };
        assert!(matches!(
            terminal.native_outcome(),
            Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
        ));
        terminal
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("close drift terminal: {cause:?}"));
    }
    let before_store = describe(&host, &reviewer, storage.branch());
    let mut changed = original.clone();
    changed.attempt = 807;
    let AuthoredIntent::Retention(edit) = &mut changed.proposal.intents[1];
    edit.days = 4;
    let request = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(storage.branch());
    let operation = storage.claim_operation(&reviewer, &changed, 807).unwrap();
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
        panic!("changed body domain terminal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            attempts::claim::Denial::ProposalChanged
        ))
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close changed body terminal: {cause:?}"));
    let mut wrong_key = original.clone();
    wrong_key.attempt = 809;
    let operation = storage.claim_operation(&reviewer, &wrong_key, 810).unwrap();
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
        panic!("key mismatch domain terminal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            attempts::claim::Denial::AttemptKeyMismatch
        ))
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close key mismatch terminal: {cause:?}"));
    assert_eq!(describe(&host, &reviewer, storage.branch()), before_store);
    assert_eq!(model.describe(&reviewer).unwrap(), source);
    let drift = storage
        .claim(&model, &author, &loaded, 804)
        .err()
        .expect("same key is not actor-local");
    let super::super::application::storage::ClaimFailure::Terminal(terminal) = drift else {
        panic!("cross-principal drift terminal: {drift:?}")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
    let author_request = host
        .request(&author.principal, author.scope)
        .on_branch(storage.branch());
    terminal
        .close(&author_request)
        .unwrap_or_else(|(_, cause)| panic!("close cross-principal terminal: {cause:?}"));
    let another = storage.claim(&model, &reviewer, &loaded, 805).unwrap();
    let another_claim = another.reference().clone();
    drop(another);
    let adopted = adopt(&mut model, &storage, &reviewer, &loaded, 804);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    let storage_before = describe(&host, &reviewer, storage.branch());
    let mismatch =
        attempts::recording::RecordResult::from_adopted(&another_claim, &adopted).unwrap_err();
    assert!(matches!(
        mismatch,
        attempts::recording::CorrespondenceFailure::AdoptionMismatch
    ));
    assert_eq!(describe(&host, &reviewer, storage.branch()), storage_before);
    let result = storage
        .record_result(&model, &reviewer, &loaded, &claim, &adopted)
        .unwrap();
    assert_eq!(
        result.steps,
        vec![
            StoredStepOutcome::Fresh { days: 2 },
            StoredStepOutcome::Fresh { days: 3 }
        ]
    );
    let duplicate = attempts::recording::RecordResult::from_adopted(&claim, &adopted).unwrap();
    let operation = storage
        .result_operation(&reviewer, &duplicate, 806)
        .unwrap();
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
        panic!("duplicate result domain terminal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            attempts::recording::Denial::AlreadyRecorded
        ))
    ));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close duplicate result terminal: {cause:?}"));
    let performed = adopted
        .close_previous(&reviewer)
        .unwrap_or_else(|failure| panic!("close previous model: {:?}", failure.cause));
    drop(performed);
    let head = model.branch();
    let current = model.describe(&reviewer).unwrap();
    let retry = storage.claim(&model, &reviewer, &loaded, 804).unwrap();
    assert!(matches!(
        retry.evidence(),
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    assert_eq!(retry.reference(), &claim);
    drop(retry);
    assert_eq!(
        storage
            .read_result(&model, &author, &loaded, &claim)
            .unwrap(),
        result
    );
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&reviewer).unwrap(), current);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    storage.close(&author).unwrap();
    model.close(&reviewer).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
