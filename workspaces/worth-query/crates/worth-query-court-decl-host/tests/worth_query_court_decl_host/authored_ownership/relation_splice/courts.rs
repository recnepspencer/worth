//! All splice commands, correction disclosure and body production use real Query paths.
use super::super::{
    correction_courts::{install_definition, reverse},
    coupled::multi::{create, settle, TwoProgram},
};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn relation_preimage_restores_a_mid_thread_stage_and_its_coupled_body() {
    lifecycle::<30>(30);
}
#[test]
fn relation_preimage_requires_the_authored_edits_original_relation_reads() {
    lifecycle::<30>(31);
}
#[test]
fn relation_preimage_refuses_changed_inverse_endpoints() {
    lifecycle::<30>(32);
}
#[test]
fn relation_preimage_refuses_an_omitted_inverse_link() {
    lifecycle::<30>(33);
}
#[test]
fn relation_preimage_refuses_extra_inverse_links() {
    lifecycle::<30>(34);
}
#[test]
fn relation_preimage_requires_removing_the_original_replacement_link() {
    lifecycle::<30>(35);
}
#[test]
fn relation_preimage_refuses_removing_a_foreign_unchanged_link() {
    lifecycle::<30>(36);
}
#[test]
fn relation_preimage_accepts_an_observed_empty_locus() {
    lifecycle::<30>(37);
}
#[test]
fn relation_preimage_refuses_an_unobserved_empty_locus() {
    lifecycle::<30>(38);
}
#[test]
fn relation_preimage_excludes_the_coupled_producers_own_same_kind_edges() {
    lifecycle::<40>(30);
}
#[test]
fn relation_preimage_refuses_unsupported_audit_retirement_before_the_authored_edit() {
    lifecycle::<30>(41);
}
#[test]
fn relation_preimage_obeys_the_installed_material_byte_bound() {
    lifecycle::<41>(42);
}
fn lifecycle<const MODE: u8>(case: u8) {
    type S<const M: u8> = Model<M>;
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    if matches!(case, 36..=38) {
        let outcome = request
            .advanced()
            .mutate(Create {
                create_co: true,
                co_anchors: vec![20, 30],
                scope: 1,
                id: 10,
                depth: if case == 36 { 2 } else { 0 },
            })
            .without_source()
            .idempotency(&"body-and-stages".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
    } else {
        create(&host, &request, 10, vec![20, 30]);
    }
    settle(&host, &request, 10);
    let original_edges: &[&str] = if matches!(case, 37 | 38) {
        &[]
    } else if case == 36 {
        &["10->30", "20->10", "30->20"]
    } else {
        &["20->10", "30->20"]
    };
    let original_subjects =
        inspection::inspect(&host, &request, original_edges, "original-stage-links");
    super::super::coupled::producer::handler::reset_contacts();
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: case,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 20,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"splice-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    if matches!(case, 31 | 38 | 41 | 42) {
        require_relation_refusal(
            outcome,
            if case == 42 {
                WorthQueryRelationPreImageDenial::ExceedsByteBound
            } else if case == 38 {
                WorthQueryRelationPreImageDenial::MissingMutation
            } else if case == 41 {
                WorthQueryRelationPreImageDenial::UnsupportedVisibility
            } else {
                WorthQueryRelationPreImageDenial::MissingObservation
            },
            if case == 38 {
                vec![original_subjects
                    .iter()
                    .find(|(name, _)| name == "@20")
                    .unwrap()
                    .1
                    .clone()]
            } else {
                vec![]
            },
        );
        inspection::inspect(&host, &request, original_edges, "failed-splice-links");
        for id in [20, 101, 102] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
        assert!(request.query(query::Input { id: 103 }).execute().is_err());
        return;
    }
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: retired, ..
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    let old = retired
        .outputs_of::<RetiredOutputs>()
        .unwrap()
        .entity::<Retired<S<MODE>>>()
        .unwrap()
        .entity_id();
    assert_eq!(super::super::coupled::producer::handler::contacts(), 1);
    let spliced_edges: &[&str] = if case == 37 {
        &[]
    } else if case == 36 {
        &["10->30", "30->10"]
    } else {
        &["30->10"]
    };
    let spliced_subjects =
        inspection::inspect(&host, &request, spliced_edges, "spliced-stage-links");
    assert!(request.query(query::Input { id: 20 }).execute().is_err());
    if matches!(case, 32..=36) {
        let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
        let selected = request.corrections().select(&retired, limits).unwrap();
        let prepared = request
            .corrections()
            .reverse(&selected)
            .idempotency("invalid-splice-inverse")
            .limits(limits)
            .prepare()
            .unwrap();
        let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
            panic!("inverse must prepare")
        };
        require_relation_refusal(
            prepared.publish().unwrap(),
            if case == 34 {
                WorthQueryRelationPreImageDenial::ExtraInverse
            } else if case == 36 {
                WorthQueryRelationPreImageDenial::ForeignEndpoint
            } else {
                WorthQueryRelationPreImageDenial::IncompleteInverse
            },
            if case == 34 {
                vec![]
            } else {
                let subjects = if matches!(case, 35 | 36) {
                    &spliced_subjects
                } else {
                    &original_subjects
                };
                subjects
                    .iter()
                    .filter(|(name, _)| {
                        if case == 35 {
                            name == "30->10"
                        } else if case == 36 {
                            name == "10->30"
                        } else {
                            !name.starts_with('@')
                        }
                    })
                    .map(|(_, record)| record.clone())
                    .collect()
            },
        );
        inspection::inspect(&host, &request, spliced_edges, "failed-inverse-stage-links");
        assert!(request.query(query::Input { id: 20 }).execute().is_err());
        assert!(request.query(query::Input { id: 103 }).execute().is_ok());
        assert!(request.query(query::Input { id: 104 }).execute().is_err());
        return;
    }
    let restored = reverse(&request, &retired, "restore-spliced-stage");
    assert_ne!(
        old,
        restored
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S<MODE>>>()
            .unwrap()
            .entity_id()
    );
    inspection::inspect(&host, &request, original_edges, "restored-stage-links");
    assert_eq!(super::super::coupled::producer::handler::contacts(), 2);
    settle(&host, &request, 10);
    assert_eq!(
        super::super::coupled::producer::handler::contacts(),
        2,
        "one body publication per splice and restoration"
    );
}
fn require_relation_refusal<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
    expected: WorthQueryRelationPreImageDenial,
    subjects: Vec<worth_relational::facade::transactions::RecordRef>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let failure = denial
        .relation_preimage_failure()
        .unwrap_or_else(|| panic!("original typed relation preimage cause: {denial:?}"));
    assert_eq!(failure.kind(), expected);
    assert!(
        !failure.affected_records().is_empty(),
        "typed refusal names its actual relation subject"
    );
    for record in failure.affected_records() {
        if subjects.is_empty() {
            assert!(
                matches!(
                    record,
                    worth_relational::facade::transactions::RecordRef::Relation(_)
                ),
                "relation failure must not substitute its authored anchor: {record:?}"
            );
        } else {
            assert!(subjects.contains(record), "actual relation read or empty-locus anchor is the subject: {record:?} vs {subjects:?}");
        }
    }
}

#[test]
fn relation_preimage_installation_requires_the_source_operations_relation_read() {
    type S = Model<39>;
    let schema = S::declaration().unwrap();
    let result = in_memory_program(
        ApplicationProgramAuthoring::<S, TwoProgram>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(actor()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        },
    );
    let error = result
        .err()
        .expect("source relation preimage lacks its own declared read");
    match error {
        WorthQueryInMemoryApplicationDenial::Contributions(cause) => {
            assert_eq!(
                cause.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::BindingNotInstalled
            );
            assert!(cause.subject().contains("RetireOp"), "{cause:?}");
        }
        other => panic!("unrelated installation refusal: {other:?}"),
    }
}
