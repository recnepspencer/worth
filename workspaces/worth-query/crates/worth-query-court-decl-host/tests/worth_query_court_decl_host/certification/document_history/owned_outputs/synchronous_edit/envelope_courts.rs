//! All authored part values and required roots publish in one transaction.
use super::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;
fn exercise(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
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
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let envelope = request
        .edit_envelope()
        .stage::<fixture::ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .stage::<fixture::MemberTree, _>(fixture::member::Intent {
            key: "m".into(),
            value: 6,
            key_derived_target: false,
        })
        .stage::<fixture::CapTree, _>(fixture::cap::Intent {
            key: "envelope-cap".into(),
            value: 8,
            key_derived_target: false,
        })
        .stage::<fixture::CalibrationTree, _>(fixture::calibration::Intent {
            key: "envelope-calibration".into(),
            value: 10,
            key_derived_target: false,
        })
        .idempotency("four-parts".to_owned());
    let model = envelope.stage_token::<fixture::model::Binding>(0).unwrap();
    let member = envelope.stage_token::<fixture::member::Binding>(1).unwrap();
    let cap = envelope.stage_token::<fixture::cap::Binding>(2).unwrap();
    let calibration = envelope
        .stage_token::<fixture::calibration::Binding>(3)
        .unwrap();
    let before = courts::native_head(&host).0;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, published) = observe_maintenance_work(8, || {
        envelope
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        1
    );
    let Outcome::Committed(receipt) = outcome else {
        panic!("four authored parts did not commit")
    };
    assert_eq!(receipt.stage_count(), 4);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "shared automatic roots coalesce to exactly two producer routes"
    );
    assert_eq!(courts::native_head(&host).1, [before]);
    for (ordinal, old, expected, result) in [
        (0, 2, 3, receipt.result(&model).unwrap()),
        (1, 5, 6, receipt.result(&member).unwrap()),
        (2, 7, 8, receipt.result(&cap).unwrap()),
        (3, 9, 10, receipt.result(&calibration).unwrap()),
    ] {
        assert_eq!(
            (
                result.ordinal,
                result.old_value,
                result.value,
                result.key.as_str()
            ),
            (ordinal, old, expected, "four-parts")
        );
    }
    for (ordinal, identity, stage_receipt) in [
        (
            0,
            receipt.stage_identity(&model).unwrap(),
            receipt.stage_receipt(&model).unwrap(),
        ),
        (
            1,
            receipt.stage_identity(&member).unwrap(),
            receipt.stage_receipt(&member).unwrap(),
        ),
        (
            2,
            receipt.stage_identity(&cap).unwrap(),
            receipt.stage_receipt(&cap).unwrap(),
        ),
        (
            3,
            receipt.stage_identity(&calibration).unwrap(),
            receipt.stage_receipt(&calibration).unwrap(),
        ),
    ] {
        assert_eq!(identity.ordinal(), ordinal);
        assert_eq!(
            stage_receipt.commit_reference(),
            receipt.receipt().commit_reference()
        );
    }
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3
    );
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        30
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1003
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let head = courts::native_head(&host);
    let canonical_keys = host.coupled_producer_keys_on_this_thread_for_test();
    assert_eq!(canonical_keys.len(), 2);
    let (_, published) = observe_maintenance_work(8, || {
        macro_rules! settle {
            ($family:ident, $root:ty) => {{
                let mut operation = request
                    .demand($family::Demand("a".into()))
                    .retain_program_outputs::<fixture::Program, $root>(&host)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                loop {
                    match operation.advance(&request) {
                        WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                        WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                            settled.deliver_current(&request).unwrap();
                            let expected = canonical_keys.iter().find(|(producer, _, _)|
                                *producer == $family::initial::Producer::IDENTITY
                            ).expect("the real canonical owner observed this producer");
                            let receipt = settled.settlement().root_receipt()
                                .expect("the actual completed coupled producer receipt");
                            assert_eq!(receipt.idempotency_binding().key_identity(), &expected.2,
                                "the committed producer key equals its input-derived canonical owner output");
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
            }};
        }
        settle!(first, fixture::ModelTree);
        settle!(second, fixture::MemberTree);
    });
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "fresh demands reuse input-derived producer receipts"
    );
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        0
    );
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn four_authored_parts_have_one_commit_and_their_own_results() {
    exercise(false);
}
#[test]
fn four_authored_parts_have_one_commit_and_their_own_results_worker() {
    exercise(true);
}
