//! Spawns commit, retain their link and reverse through the real correction path.
use super::world::{install, Host};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
type Request<'a, 'b, 'c> = WorthQueryApplicationRequest<'a, 'b, 'c, Schema>;
fn spawn(
    host: &Host,
    request: &Request<'_, '_, '_>,
    input: Spawn,
) -> WorthQueryApplicationMutationOutcome<Refused, Done> {
    let key = format!("spawn:{}", input.id);
    request
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
fn spawned(
    host: &Host,
    request: &Request<'_, '_, '_>,
    input: Spawn,
) -> WorthQueryApplicationCommitReceipt {
    match spawn(host, request, input) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        outcome => panic!("{outcome:?}"),
    }
}
fn input(id: u64, part: bool, inverse_fault: u8) -> Spawn {
    Spawn {
        id,
        part,
        link: true,
        inverse_fault,
    }
}
fn live(request: &Request<'_, '_, '_>, id: u64) -> bool {
    match request.query(query::Read { id }).execute() {
        Ok(rows) => rows.rows().iter().any(|row| row.id == id),
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_)) => false,
        Err(cause) => panic!("{id}: {cause:?}"),
    }
}
fn has_part(receipt: &WorthQueryApplicationCommitReceipt) -> bool {
    receipt
        .outputs_of::<SpawnOutputs>()
        .unwrap()
        .entity::<SpawnedPart>()
        .unwrap()
        .is_some()
}
fn reversal<'a>(
    request: &'a Request<'_, '_, '_>,
    original: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial> {
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request
        .corrections()
        .select_recorded(&original.correction_source(), limits)
        .unwrap();
    request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
}
fn reverse(
    request: &Request<'_, '_, '_>,
    original: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let Ok(WorthQueryApplicationCorrectionPreparation::Prepared(prepared)) =
        reversal(request, original, key)
    else {
        panic!("{key}: the relation-only inverse must derive and prepare")
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{key}: {outcome:?}"
    );
}
fn relation_refusal<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) -> WorthQueryRelationPreImageFailure {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    denial
        .relation_preimage_failure()
        .unwrap_or_else(|| panic!("typed relation pre-image refusal: {denial:?}"))
        .clone()
}
fn world() -> (Host, WorthQueryRequestScope) {
    (install(), request_scope())
}
#[test]
fn extension_relation_inverse_reverses_a_spawn_that_bore_its_optional_part() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = spawned(&host, &request, input(10, true, 0));
    assert!(has_part(&original));
    assert!(live(&request, 10) && live(&request, 11));
    reverse(&request, &original, "despawn-with-part");
    assert!(!live(&request, 10), "spawned node retired");
    assert!(
        !live(&request, 11),
        "the born part is read as present and retired"
    );
    assert!(live(&request, SCOPE), "the scope survives");
}
#[test]
fn relation_only_inverse_reverses_a_spawn_whose_optional_part_was_absent() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    // A live unrelated node carries the id a misread present part would name.
    spawned(&host, &request, input(11, false, 0));
    let original = spawned(&host, &request, input(10, false, 0));
    assert!(!has_part(&original));
    reverse(&request, &original, "despawn-without-part");
    assert!(!live(&request, 10), "spawned node retired");
    assert!(live(&request, 11), "an absent part is read as absent");
    assert!(live(&request, SCOPE), "the scope it held survives");
}
#[test]
fn relation_only_inverse_refuses_an_inverse_that_removes_a_foreign_link() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    spawned(&host, &request, input(11, false, 0));
    let original = spawned(&host, &request, input(10, false, 1));
    let Ok(WorthQueryApplicationCorrectionPreparation::Prepared(prepared)) =
        reversal(&request, &original, "despawn-with-foreign-link")
    else {
        panic!("the inverse derives and prepares before its commit refuses")
    };
    let failure = relation_refusal(prepared.publish().unwrap());
    assert_eq!(
        failure.kind(),
        WorthQueryRelationPreImageDenial::ForeignEndpoint
    );
    assert!(!failure.affected_records().is_empty(), "{failure:?}");
    for record in failure.affected_records() {
        assert!(
            matches!(
                record,
                worth_relational::facade::transactions::RecordRef::Relation(_)
            ),
            "the refusal names the foreign link: {record:?}"
        );
    }
    assert!(
        live(&request, 10) && live(&request, 11),
        "nothing was retired"
    );
}
#[test]
fn relation_only_inverse_discloses_no_undeclared_relation_material() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = spawned(&host, &request, input(10, true, 2));
    let refused = reversal(&request, &original, "despawn-reading-unrelated");
    assert!(
        matches!(
            &refused,
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Domain(
                    ApplicationCorrectionDerivationDenial::MissingPreImage
                )
            ))
        ),
        "{:?}",
        refused.err()
    );
    assert!(
        live(&request, 10) && live(&request, 11),
        "nothing was retired"
    );
}
#[test]
fn relation_only_spawn_without_its_declared_link_refuses_before_commit() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let unlinked = Spawn {
        link: false,
        ..input(10, true, 0)
    };
    let failure = relation_refusal(spawn(&host, &request, unlinked));
    assert_eq!(
        failure.kind(),
        WorthQueryRelationPreImageDenial::MissingMutation
    );
    assert!(
        !live(&request, 10) && !live(&request, 11),
        "nothing committed"
    );
}

#[test]
fn relation_only_edit_seals_unchanged_demanded_kinds_and_restores_the_changed_pair() {
    let (host, scope) = world();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    spawned(&host, &request, input(10, true, 0));
    spawned(&host, &request, input(20, false, 0));
    let edit = |key: &str| {
        let outcome = request
            .advanced()
            .mutate(super::retarget::Input {
                id: 10,
                old: 11,
                target: 20,
            })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        match outcome {
            WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
            outcome => panic!("the relation-only edit commits: {outcome:?}"),
        }
    };
    let original = edit("retarget-first");
    reverse(&request, &original, "restore-held-pair");
    // This fresh edit's decision reads the actual edge restored by the inverse.
    // It refuses unless 10 holds 11 again; this also catches a wrong inverse.
    edit("retarget-after-undo");
    assert!(live(&request, 10) && live(&request, 11) && live(&request, 20));
}
