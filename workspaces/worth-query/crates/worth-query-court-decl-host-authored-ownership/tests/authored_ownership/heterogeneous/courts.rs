//! Public declaration and installed co-anchor retirement use actual typed kinds.
use super::super::{
    correction_courts::{install_program, reverse},
    program::*,
};
use super::*;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchemaDenialKind;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<44, Present<true>>;
#[test]
fn heterogeneous_co_anchors_restore_both_logical_keys_and_fresh_generations() {
    run(false);
}
#[test]
fn heterogeneous_co_anchor_key_corruption_refuses_without_effect() {
    run(true);
}
fn run(corrupt: bool) {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = request
        .advanced()
        .mutate(Create {
            create_co: true,
            co_anchors: vec![],
            scope: 1,
            id: 10,
            depth: 0,
        })
        .without_source()
        .idempotency(&"hetero-birth".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = created
    else {
        panic!("{created:?}")
    };
    super::super::co_courts::produce(&host, &request, 10);
    let old_source = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let old = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<BodyCreated<S>>()
        .unwrap()
        .entity_id();
    let retire = |nonce: &str| {
        let outcome = request
            .advanced()
            .mutate(Retire {
                relation_fault: 0,
                corrupt_restoration: corrupt,
                co_anchors: vec![],
                scope: 1,
                id: 10,
                descendants: vec![101, 102],
            })
            .without_source()
            .idempotency(&nonce.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        receipt
    };
    let retired = retire("hetero-retire");
    if corrupt {
        let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
        let selected = request.corrections().select(&retired, limits).unwrap();
        let prepared = request
            .corrections()
            .reverse(&selected)
            .idempotency("bad-body-key")
            .limits(limits)
            .prepare();
        let outcome = match prepared.expect("retained heterogeneous fields are valid") {
            WorthQueryApplicationCorrectionPreparation::Prepared(prepared) => {
                prepared.publish().unwrap()
            }
            WorthQueryApplicationCorrectionPreparation::Settled(outcome) => outcome,
            _ => panic!("corrupt inverse must finish with an identity refusal"),
        };
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::CorrectionIdentityMismatch
        );
        for id in [10, 101, 102] {
            assert!(request
                .query(super::super::query::Input { id })
                .execute()
                .is_err());
        }
        for id in [10010, 10011] {
            assert!(request.query(query::Input { id }).execute().is_err());
        }
        return;
    }
    let restored = reverse(&request, &retired, "hetero-restore");
    let fresh_source = restored
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    assert_ne!(old_source, fresh_source);
    let fresh = restored
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<BodyCreated<S>>()
        .unwrap()
        .entity_id();
    assert_ne!(old, fresh);
    assert_eq!(
        request
            .query(query::Input { id: 10010 })
            .execute()
            .unwrap()
            .rows()[0]
            .id,
        10010
    );
    assert!(request
        .query(super::super::query::Input { id: 10 })
        .execute()
        .is_ok());
    // The ordinary typed reader must resolve the exact retained BodyKey again.
    let again = retire("hetero-retire-again");
    assert_eq!(
        again
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<BodyRetired<S>>()
            .unwrap()
            .entity_id(),
        fresh
    );
}
#[test]
fn heterogeneous_non_anchor_kind_refuses_with_its_typed_subject() {
    let error = super::super::correction_courts::try_install_definition::<
        Model<45, Present<false>>,
        Program,
    >()
    .err()
    .expect("Body is not declared as an anchor by this correction");
    let WorthQueryInMemoryApplicationDenial::Schema(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::PreImageDemandNotCoveredByAuthoredAnchors
    );
    assert_eq!(error.subject(), "Body");
}

#[test]
fn a_co_anchor_declared_by_a_different_operation_cannot_cover_the_retirement_demand() {
    let error = super::super::correction_courts::try_install_definition::<
        Model<46, Present<true>>,
        Program,
    >()
    .err()
    .expect("Create's co-anchor cannot cover Retire's pre-image");
    let WorthQueryInMemoryApplicationDenial::Schema(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::PreImageDemandNotCoveredByAuthoredAnchors
    );
    assert_eq!(error.subject(), "Body");
}
