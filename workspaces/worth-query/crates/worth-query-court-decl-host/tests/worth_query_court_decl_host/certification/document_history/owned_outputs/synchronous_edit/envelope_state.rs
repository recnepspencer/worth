//! Envelope drift and failures preserve their exact original transaction boundary.
use super::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
type Host = WorthQueryProgramApplicationRuntime<Schema, fixture::Program>;
fn host(worker: bool) -> Host {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    world::install_configured(
        limits,
        2,
        None,
        world::Configuration {
            envelope_records: true,
            ..Default::default()
        },
    )
}
macro_rules! two {
    ($request:expr,$member:expr,$key:expr) => {
        $request
            .edit_envelope()
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "m".into(),
                value: $member,
                key_derived_target: false,
            })
            .idempotency($key.to_owned())
    };
}
fn drift(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let Outcome::Committed(receipt) = two!(request, 6, "drift")
        .execute_performed::<fixture::Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("live envelope refused")
    };
    let before = courts::native_head(&host);
    let (replay, writes) = observe_maintenance_work(8, || {
        two!(request, 6, "drift")
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::AlreadyCommitted(replay) = replay else {
        panic!("whole envelope was not replayed")
    };
    assert_eq!(
        replay.commit_reference(),
        receipt.receipt().commit_reference()
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    let (changed, writes) = observe_maintenance_work(8, || {
        two!(request, 7, "drift")
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert!(matches!(changed, Outcome::IdempotencyIntentDrift));
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    let (swapped, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "m".into(),
                value: 6,
                key_derived_target: false,
            })
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .idempotency("drift".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert!(matches!(swapped, Outcome::IdempotencyIntentDrift));
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), before);
}
fn refusal(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let envelope = two!(request, 1001, "domain-refusal");
    let token = envelope.stage_token::<fixture::member::Binding>(1).unwrap();
    let (outcome, writes) = observe_maintenance_work(8, || {
        envelope
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::DomainDenied(denial) = outcome else {
        panic!("the stage's domain denial was not retained")
    };
    assert_eq!(denial.identity().ordinal(), 1);
    assert_eq!(denial.binding(), fixture::member::Binding::IDENTITY);
    assert_eq!(denial.denial(&token).unwrap(), &Denial::ValueOutOfRange);
    let Some(WorthQueryApplicationUncommitted::Denied(abort)) = denial.commit_outcome() else {
        panic!("the original stage refusal owns its precise abort outcome")
    };
    assert_eq!(
        abort.coupled_producer_denial().unwrap().kind(),
        WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
            ordinal: 1,
            binding: fixture::member::Binding::IDENTITY
        }
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), before);
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
    assert_eq!(
        request
            .query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        5
    );
}
fn conflict(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let target = request
        .query(query::member_public::Read("envelope-calibration".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "m".into(),
                value: 6,
                key_derived_target: true,
            })
            .stage::<fixture::CalibrationTree, _>(fixture::calibration::Intent {
                key: "envelope-calibration".into(),
                value: 10,
                key_derived_target: true,
            })
            .idempotency("envelope-calibration".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("shared-key record collision was not a typed refusal")
    };
    let denial = denial
        .coupled_producer_denial()
        .expect("actual authored conflict subject");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ConflictingIntent {
            target: worth_relational::facade::transactions::RecordRef::Entity(target),
            earlier: 1,
            later: 2
        }
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), before);
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
    assert_eq!(
        request
            .query(query::member_public::Read("envelope-calibration".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        8
    );
}
#[test]
fn full_envelope_input_and_order_drift_refuse_without_publication() {
    drift(false);
}
#[test]
fn full_envelope_input_and_order_drift_refuse_without_publication_worker() {
    drift(true);
}
#[test]
fn refusing_stage_keeps_its_typed_domain_subject_and_publishes_nothing() {
    refusal(false);
}
#[test]
fn refusing_stage_keeps_its_typed_domain_subject_and_publishes_nothing_worker() {
    refusal(true);
}
#[test]
fn different_fields_of_one_record_refuse_and_name_both_stages() {
    conflict(false);
}
#[test]
fn different_fields_of_one_record_refuse_and_name_both_stages_worker() {
    conflict(true);
}

fn missing_stage_scope(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = failure::checkpoint(&host);
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "absent-scope".into(),
                value: 6,
                key_derived_target: false,
            })
            .idempotency("missing-stage-scope".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
    });
    let Err(denial) = outcome else {
        panic!("a nonexistent C scope must refuse admission")
    };
    assert_eq!(
        denial.subject(),
        worth_query_host::facade::application_entry::WorthQueryEditEnvelopeSubject::Stage {
            ordinal: 1,
            binding: fixture::member::Binding::IDENTITY
        }
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn admission_refusal_names_its_authored_stage() {
    missing_stage_scope(false);
}
#[test]
fn admission_refusal_names_its_authored_stage_worker() {
    missing_stage_scope(true);
}

fn execution_subject(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = failure::checkpoint(&host);
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "m".into(),
                value: 6,
                key_derived_target: true,
            })
            .idempotency("no-such-domain-record".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::StageExecutionDenied(denial) = outcome else {
        panic!("the failed D read must retain its stage and original typed execution cause")
    };
    assert_eq!(denial.identity().ordinal(), 1);
    assert_eq!(denial.binding(), fixture::member::Binding::IDENTITY);
    assert!(matches!(
        denial.cause(),
        worth_query_host::facade::application_entry::AuthoredStageExecutionCause::Request(
            worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(_)
        )
    ));
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn execution_refusal_retains_the_original_stage_cause() {
    execution_subject(false);
}
#[test]
fn execution_refusal_retains_the_original_stage_cause_worker() {
    execution_subject(true);
}
