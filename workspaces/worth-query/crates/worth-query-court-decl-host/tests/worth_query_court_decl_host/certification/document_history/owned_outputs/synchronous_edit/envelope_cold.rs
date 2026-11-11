//! Real cold symbol pages preserve all named rows and the whole envelope meaning.
use super::envelope_fixture as fixture;
use super::*;
mod row_correspondence;
mod termination;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::{
    inspection::observe_maintenance_work, publication::RecordStructuralChange,
    transactions::RecordRef,
};
fn exercise(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let mut runs = Vec::new();
    for cold in [false, true] {
        let limits = if worker {
            world::worker_limits()
        } else {
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
        };
        let host: WorthQueryProgramApplicationRuntime<Schema, fixture::Program> =
            world::install_configured(
                limits,
                2,
                None,
                world::Configuration {
                    envelope_records: true,
                    ..Default::default()
                },
            );
        let placement = cold.then(|| {
            host.with_host_advancement(|phase| {
                host.runtime()
                    .place_symbol_pages_cold_for_test(&phase)
                    .unwrap()
            })
            .unwrap()
        });
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        macro_rules! build {
            ($request:expr) => {
                ($request)
                    .edit_envelope()
                    .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                        key: "a".into(),
                        value: 3,
                        key_derived_target: false,
                    })
                    .stage::<fixture::RestoreTree, _>(authored_member::restore::Intent {
                        key: "envelope-born".into(),
                        value: 11,
                    })
                    .idempotency("cold-meaning".to_owned())
            };
        }
        let envelope = build!(request);
        let model = envelope.stage_token::<fixture::model::Binding>(0).unwrap();
        let restored = envelope
            .stage_token::<authored_member::restore::Binding>(1)
            .unwrap();
        let (outcome, writes) = observe_maintenance_work(8, || {
            envelope
                .execute_performed::<fixture::Program>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        let Outcome::Committed(receipt) = outcome else {
            panic!("cold={cold}: the genuine envelope refused: {outcome:?}")
        };
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        if let Some(placement) = placement {
            assert!(
                placement.page_reads_for_test() > 0,
                "real bucket pages were admitted, not a simulated retry"
            );
        }
        let source = request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let member = request
            .query(query::member_public::Read("envelope-born".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let first = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let rows = [source, member, first]
            .into_iter()
            .map(|row| (row.key, (row.native_id, row.value)))
            .collect::<BTreeMap<_, _>>();
        let created = receipt
            .stage_receipt(&restored)
            .unwrap()
            .outputs_of::<authored_member::restore::Outputs>()
            .unwrap()
            .entity::<authored_member::restore::Restored>()
            .unwrap()
            .entity_id();
        assert_eq!(
            created, rows["envelope-born"].0,
            "the suffix's real allocation, not an unscoped local client key"
        );
        let changes = receipt.receipt().committed_changes();
        let (published, metadata) = row_correspondence::observe(
            changes.record_patches_for_test(),
            &rows,
            receipt.receipt().idempotency_binding().key_identity(),
            receipt.receipt().changed_record_count(),
        );
        assert_eq!(
            metadata.len(),
            2,
            "one envelope and one input-derived producer receipt"
        );
        let result = receipt.result(&model).unwrap().clone();
        let identities = [
            *receipt.stage_identity(&model).unwrap(),
            *receipt.stage_identity(&restored).unwrap(),
        ];
        let counts = (
            receipt.receipt().changed_record_count(),
            receipt.receipt().emitted_effect_count(),
        );
        eprintln!("cold={cold} named rows={rows:?}; original patches={:?}; lineage={:?}; exact sessions, witnesses and IDs={:?}; stage receipt={:?}", changes.record_patches_for_test(), changes.lineage_events(), receipt.receipt(), receipt.stage_receipt(&restored).unwrap());
        let head = courts::native_head(&host);
        let (replay, writes) = observe_maintenance_work(8, || {
            build!(request)
                .execute_performed::<fixture::Program>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        let Outcome::AlreadyCommitted(replayed) = replay else {
            panic!("cold={cold}: replay was not committed")
        };
        assert_eq!(
            replayed.commit_reference(),
            receipt.receipt().commit_reference()
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(courts::native_head(&host), head);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (_, writes) = observe_maintenance_work(8, || {
            let mut operation = request
                .demand(first::Demand("a".into()))
                .retain_program_outputs::<fixture::Program, fixture::ModelTree>(&host)
                .unwrap_or_else(|e| panic!("{:?}", e.denial()));
            loop {
                match operation.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                    WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                        settled.deliver_current(&request).unwrap();
                        settled
                            .close(&request)
                            .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                        break;
                    }
                    WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                        panic!("{denial:?}")
                    }
                    WorthQueryProgramOutputOperationAdvance::Terminal(t) => {
                        panic!("{:?}", t.denial())
                    }
                }
            }
        });
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts,
            "both warm and cold producer receipts retain their input-derived meaning"
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(courts::native_head(&host), head);
        let live_commit = receipt.receipt().commit_reference().commit_id;
        let checkpoint = failure::checkpoint(&host);
        let expected_rows = rows.clone();
        runs.push((
            rows,
            published,
            result,
            identities,
            counts,
            metadata,
            changes.lineage_events().to_vec(),
        ));
        drop(replayed);
        drop(receipt);
        drop(principal);
        drop(scope);
        drop(host);
        let restored =
            world::restore_program_checkpoint::<fixture::Program>(checkpoint, worker, false);
        let scope = request_scope();
        let principal = authenticate_operator(restored.installed_schema(), &scope);
        let request = restored.request(&principal, &scope);
        for row in [
            request
                .query(query::model::Read("a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
            request
                .query(query::member_public::Read("envelope-born".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
        ] {
            assert_eq!(
                expected_rows[&row.key],
                (row.native_id, row.value),
                "cold reopen preserves each named row's actual identity"
            );
        }
        let contacts = restored.producer_contacts_on_this_thread_for_test();
        let head = courts::native_head(&restored);
        let (replay, writes) = observe_maintenance_work(8, || {
            build!(request)
                .execute_performed::<fixture::Program>(
                    &restored,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        let Outcome::PreviouslyCommitted(recorded) = replay else {
            panic!(
                "reopen without a retained live receipt reports its historical commit: {replay:?}"
            )
        };
        assert_eq!(recorded.commit_id(), live_commit);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        let (drift, writes) = observe_maintenance_work(8, || {
            request
                .edit_envelope()
                .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: 3,
                    key_derived_target: false,
                })
                .stage::<fixture::RestoreTree, _>(authored_member::restore::Intent {
                    key: "envelope-born".into(),
                    value: 12,
                })
                .idempotency("cold-meaning".to_owned())
                .execute_performed::<fixture::Program>(
                    &restored,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        assert!(
            matches!(drift, Outcome::IdempotencyIntentDrift),
            "a different reopened envelope is drift, not a historical replay: {drift:?}"
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(
            restored.producer_contacts_on_this_thread_for_test(),
            contacts
        );
        assert_eq!(courts::native_head(&restored), head);
    }
    let warm = &runs[0];
    let cold = &runs[1];
    row_correspondence::named_values(&warm.0, &cold.0);
    row_correspondence::records(&warm.1, &cold.1);
    assert_eq!((&warm.2, warm.3, warm.4), (&cold.2, cold.3, cold.4));
    assert_eq!(
        warm.5, cold.5,
        "named idempotency rows keep intent and effect meaning"
    );
    row_correspondence::lineages(
        &warm.6,
        &cold.6,
        warm.1
            .values()
            .filter(|(id, change)| {
                matches!(id, RecordRef::Entity(_)) && *change == RecordStructuralChange::Created
            })
            .count(),
    );
    first_cold_refusal(worker);
}
#[test]
fn cold_and_warm_symbol_buckets_publish_equivalent_named_rows() {
    exercise(false);
}
#[test]
fn cold_and_warm_symbol_buckets_publish_equivalent_named_rows_worker() {
    exercise(true);
}

fn first_cold_refusal(worker: bool) {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, fixture::Program> =
        world::install_configured(
            limits,
            2,
            None,
            world::Configuration {
                envelope_records: true,
                ..Default::default()
            },
        );
    let placement = host
        .with_host_advancement(|phase| {
            host.runtime()
                .place_symbol_pages_cold_for_test(&phase)
                .unwrap()
        })
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let envelope = request
        .edit_envelope()
        .stage::<fixture::ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .stage::<fixture::MemberTree, _>(fixture::member::Intent {
            key: "m".into(),
            value: 1001,
            key_derived_target: false,
        })
        .idempotency("first-cold-refusal".to_owned());
    let token = envelope.stage_token::<fixture::member::Binding>(1).unwrap();
    let (outcome, writes) = observe_maintenance_work(2048, || {
        envelope
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::DomainDenied(denial) = outcome else {
        panic!("the first cold-attempt refusal was replaced")
    };
    termination::assert_refusal_stops(&writes);
    assert!(
        placement.page_reads_for_test() > 0,
        "the refusing attempt must read a genuine cold symbol page"
    );
    assert_eq!(denial.binding(), fixture::member::Binding::IDENTITY);
    assert_eq!(denial.identity().ordinal(), 1);
    assert_eq!(denial.denial(&token).unwrap(), &Denial::ValueOutOfRange);
    let Some(WorthQueryApplicationUncommitted::Denied(abort)) = denial.commit_outcome() else {
        panic!("cold stage refusal lost its abort custody")
    };
    assert_eq!(
        abort.coupled_producer_denial().unwrap().kind(),
        WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
            ordinal: 1,
            binding: fixture::member::Binding::IDENTITY
        }
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), head);
    failure::assert_model_unchanged(&host, &before);
    eprintln!(
        "first cold-attempt denial={:?}, page reads={}",
        denial.commit_outcome(),
        placement.page_reads_for_test()
    );
}
