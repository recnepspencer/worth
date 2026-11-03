//! Co-anchors are explicit authored sources, with separate inverse Creates.
use super::{
    co_roles::*,
    correction_courts::{install_program, reverse},
    lifecycle::*,
    program::*,
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<5>;
type Request<'a, 'b, 'c> = WorthQueryApplicationRequest<'a, 'b, 'c, S>;
pub(super) fn create(
    host: &WorthQueryProgramApplicationRuntime<S, Program>,
    request: &Request<'_, '_, '_>,
    id: u64,
    co: Vec<u64>,
) -> WorthQueryApplicationCommitReceipt {
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: true,
            scope: 1,
            id,
            depth: 1,
            co_anchors: co,
        })
        .without_source()
        .idempotency(&format!("create:{id}"))
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
}
pub(super) fn produce(
    host: &WorthQueryProgramApplicationRuntime<S, Program>,
    request: &Request<'_, '_, '_>,
    id: u64,
) {
    let mut pending = request
        .demand(Demand(id))
        .retain_program_outputs::<Program, Graph<S>>(host)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..32 {
        match pending.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled
                    .close(request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("producer did not settle");
}
fn absent(request: &Request<'_, '_, '_>, ids: &[u64]) {
    for &id in ids {
        assert!(
            matches!(
                request.query(query::Input { id }).execute(),
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
            ),
            "{id} remains live"
        );
    }
}
#[test]
fn undo_first_stage_with_implicitly_born_owner_retires_both_authored_anchors() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    produce(&host, &request, 10);
    produce(&host, &request, 20);
    let schema = S::declaration().unwrap();
    let entries = schema.identity().canonical_basis().payload().entries();
    for expected in ["co-created-a", "co-retired-a"] {
        assert!(entries.iter().any(|entry| matches!(entry.locus(), worth_foundational::facade::CanonicalBasisLocus::Named(worth_foundational::facade::InternedString::Raw(name)) if name.ends_with(".co-anchors.0.source-role"))
            && matches!(entry.value(), worth_foundational::facade::CanonicalBasisValue::ExactText(worth_foundational::facade::InternedString::Raw(value)) if value.as_str() == expected)), "co-anchor roles are part of installed schema meaning");
    }
    let retired = reverse(&request, &created, "undo-first-stage");
    assert!(retired
        .outputs_of::<RetiredOutputs>()
        .unwrap()
        .entity::<CoRetired<S, 0>>()
        .unwrap()
        .is_some());
    absent(&request, &[10, 20, 101, 102, 201, 202]);
}
#[test]
fn whole_multi_stage_retirement_restores_every_authored_anchor_with_fresh_ids() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let first = create(&host, &request, 10, vec![]);
    let second = create(&host, &request, 20, vec![]);
    let third = create(&host, &request, 30, vec![]);
    for id in [10, 20, 30] {
        produce(&host, &request, id);
    }
    let outcome = request
        .advanced()
        .mutate(Retire {
            corrupt_restoration: false,
            scope: 1,
            id: 10,
            co_anchors: vec![20, 30],
            descendants: vec![101, 102, 201, 202, 301, 302],
        })
        .without_source()
        .idempotency(&"retire-whole".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    absent(&request, &[10, 20, 30, 101, 102, 201, 202, 301, 302]);
    let restored = reverse(&request, &receipt, "restore-whole");
    let original_id = |receipt: &WorthQueryApplicationCommitReceipt| {
        receipt
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S>>()
            .unwrap()
            .entity_id()
    };
    let roles = restored.outputs_of::<CreatedOutputs>().unwrap();
    assert_ne!(
        roles.entity::<Created<S>>().unwrap().entity_id(),
        original_id(&first)
    );
    assert_ne!(
        roles
            .entity::<CoCreated<S, 0>>()
            .unwrap()
            .unwrap()
            .entity_id(),
        original_id(&second)
    );
    assert_ne!(
        roles
            .entity::<CoCreated<S, 1>>()
            .unwrap()
            .unwrap()
            .entity_id(),
        original_id(&third)
    );
    // The nested-create decision reads the actual owned edge on the fresh co-owner.
    // Canonical claimed endpoint metadata alone cannot satisfy this read.
    let nested = request
        .advanced()
        .mutate(Create {
            scope: 20,
            id: 999,
            depth: 0,
            co_anchors: vec![],
            create_co: true,
        })
        .without_source()
        .idempotency(&"check-restored-edge".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            nested,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{nested:?}"
    );
    for id in [10, 20, 30, 101, 102, 201, 202, 301, 302] {
        assert_eq!(
            request.query(query::Input { id }).execute().unwrap().rows()[0].id,
            id
        );
    }
}
#[test]
fn undeclared_extra_authored_source_is_not_implicitly_adopted_as_a_co_anchor() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![]);
    create(&host, &request, 20, vec![]);
    produce(&host, &request, 10);
    let outcome = request
        .advanced()
        .mutate(Retire {
            corrupt_restoration: false,
            scope: 1,
            id: 10,
            co_anchors: vec![],
            descendants: vec![101, 102, 20],
        })
        .without_source()
        .idempotency(&"extra".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        denial
            .authored_retirement_error()
            .unwrap()
            .authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ExtraEntity)
    );
    for id in [10, 20, 101, 102] {
        assert_eq!(
            request.query(query::Input { id }).execute().unwrap().rows()[0].id,
            id
        );
    }
}

#[test]
fn absent_optional_birth_cannot_retire_a_preexisting_owner() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 20, vec![]);
    produce(&host, &request, 20);
    let outcome = request
        .advanced()
        .mutate(Create {
            scope: 1,
            id: 10,
            depth: 0,
            co_anchors: vec![20],
            create_co: false,
        })
        .without_source()
        .idempotency(&"existing-owner".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    assert!(receipt
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<CoCreated<S, 0>>()
        .unwrap()
        .is_none());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let source = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&source)
        .idempotency("bad-optional-inverse")
        .limits(limits)
        .prepare()
        .unwrap();
    let outcome = match preparation {
        WorthQueryApplicationCorrectionPreparation::Prepared(prepared) => {
            prepared.publish().unwrap()
        }
        WorthQueryApplicationCorrectionPreparation::Settled(outcome) => outcome,
        _ => panic!("inverse must finish with its identity refusal"),
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
    for id in [10, 20, 201, 202] {
        assert_eq!(
            request.query(query::Input { id }).execute().unwrap().rows()[0].id,
            id
        );
    }
}
