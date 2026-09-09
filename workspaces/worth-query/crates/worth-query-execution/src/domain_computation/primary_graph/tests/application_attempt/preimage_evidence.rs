//! Exact reusable commit evidence and demanded-work breadth proofs.

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::{authenticated_principal, idempotency, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, Account, AccountLabel, AccountStatus, AuthorizationWorld,
    ExactStatusRetentionInput, ExactStatusRetentionOperation, IdentityExecutionSchema, Principal,
    RetainedStatusEffect, RetainedStatusNotice,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
};

#[test]
fn response_loss_and_interleaving_preserve_one_preimage_and_outbox_bundle() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let unrelated = resolved_account(&world, "unrelated", &request);
    let selected = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commit_count();
    let first = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "frozen",
        RetentionMutationBreadth::Narrow,
    );
    let retry = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "frozen",
        RetentionMutationBreadth::Narrow,
    );
    let interleaved = retained_status_program(
        &world,
        &principal,
        &unrelated,
        &request,
        "changed-between",
        RetentionMutationBreadth::Narrow,
    );

    world.faults.lose_next_commit_response();
    let outcome = world
        .application
        .compare_and_commit_application(first, idempotency(81, 82));
    let WorthQueryApplicationCommitOutcome::Committed(original) = outcome else {
        panic!("response-loss recovery must return the authoritative commit: {outcome:?}");
    };
    let after_original = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_ne!(after_original.selected_commit(), selected.selected_commit());
    assert_eq!(commit_count(), baseline + 1);
    let outcome = world
        .application
        .compare_and_commit_application(interleaved, idempotency(83, 84));
    super::assert_product_basis_stale(
        outcome,
        "the preadmitted independent interleave bound to the prior product",
    );
    assert_eq!(commit_count(), baseline + 1);
    assert_eq!(
        world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .unwrap()
            .selected_commit(),
        after_original.selected_commit()
    );

    let readmitted_interleave = retained_status_program(
        &world,
        &principal,
        &unrelated,
        &request,
        "changed-between",
        RetentionMutationBreadth::Narrow,
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(readmitted_interleave, idempotency(83, 84)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(81, 82))
    else {
        panic!("later equivalent resolution must reopen the original evidence");
    };

    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(recovered.mutation_work(), original.mutation_work());
    assert_eq!(recovered.dispatch_outbox(), original.dispatch_outbox());
    assert_eq!(recovered.retained_preimage(), original.retained_preimage());
    assert_eq!(original.terminal().attempt_resources_released(), Some(true));
    assert_eq!(recovered.terminal().attempt_resources_released(), None);
    assert!(recovered.dispatch_outbox().is_some());
    assert_retained_status(&recovered, "open");
    assert_eq!(commit_count(), baseline + 2);
}

#[test]
fn later_commit_recovery_selects_exact_evidence_and_rejects_foreign_reference_axes() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let first_account = resolved_account(&world, "open", &request);
    let later_account = resolved_account(&world, "unrelated", &request);
    let first = retained_status_program(
        &world,
        &principal,
        &first_account,
        &request,
        "first-commit",
        RetentionMutationBreadth::Narrow,
    );
    let first_outcome = world
        .application
        .compare_and_commit_application(first, idempotency(85, 86));
    assert!(
        matches!(
            first_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "first exact-evidence commit failed: {first_outcome:?}"
    );

    let later = retained_status_program(
        &world,
        &principal,
        &later_account,
        &request,
        "later-commit",
        RetentionMutationBreadth::Narrow,
    );
    let retry = retained_status_program(
        &world,
        &principal,
        &later_account,
        &request,
        "later-commit",
        RetentionMutationBreadth::Narrow,
    );
    let WorthQueryApplicationCommitOutcome::Committed(later_receipt) = world
        .application
        .compare_and_commit_application(later, idempotency(87, 88))
    else {
        panic!("the later exact-evidence target commits");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(87, 88))
    else {
        panic!("the later commit must not select the first stored entry");
    };
    assert!(recovered.is_same_authoritative_commit(&later_receipt));
    assert_retained_status(&recovered, "unrelated");

    let mut foreign_version = later_receipt.commit_reference().clone();
    foreign_version.version_id =
        worth_relational::facade::identity::VersionId::new(foreign_version.version_id.as_u64() + 1);
    let mut foreign_branch = later_receipt.commit_reference().clone();
    foreign_branch.branch_id =
        worth_relational::facade::history::BranchId("foreign-branch".to_owned());
    let mut foreign_parents = later_receipt.commit_reference().clone();
    foreign_parents.parents.push(foreign_parents.commit_id);
    for foreign in [foreign_version, foreign_branch, foreign_parents] {
        assert!(
            world
                .application
                .primary_provider
                .observe_completed_application(&foreign)
                .is_none(),
            "equal CommitId with any foreign RelationalCommitReceipt axis must not observe evidence"
        );
    }
}

#[test]
fn demanded_work_tracks_real_multi_intent_and_target_breadth() {
    let narrow = demanded_mutation_work(RetentionMutationBreadth::Narrow, 91);
    let wide = demanded_mutation_work(RetentionMutationBreadth::CrossRecordLabel, 92);

    assert_eq!(narrow.preimage_demanded_loci_examined(), 1);
    assert_eq!(wide.preimage_demanded_loci_examined(), 1);
    assert_eq!(narrow.preimage_mutation_targets_materialized(), 1);
    assert_eq!(wide.preimage_mutation_targets_materialized(), 2);
    assert_eq!(narrow.preimage_decision_facts_examined(), 3);
    assert_eq!(wide.preimage_decision_facts_examined(), 4);
    assert_eq!(narrow.preimage_candidates_materialized(), 1);
    assert_eq!(wide.preimage_candidates_materialized(), 2);
    assert_eq!(narrow.preimage_validated_intents_examined(), 3);
    assert_eq!(wide.preimage_validated_intents_examined(), 4);
}

fn demanded_mutation_work(
    breadth: RetentionMutationBreadth,
    key: u8,
) -> crate::domain_computation::primary_graph::provider::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program =
        retained_status_program(&world, &principal, &account, &request, "frozen", breadth);
    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(key, key));
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("demanded mutation commits: {outcome:?}");
    };
    receipt.mutation_work().unwrap().clone()
}

fn assert_retained_status(
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    expected: &str,
) {
    let retained = receipt
        .retained_preimage()
        .expect("recorded inverse retains prior truth");
    assert_eq!(
        retained
            .field_for(AccountStatus::reference())
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from(expected)
        )
    );
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum RetentionMutationBreadth {
    Narrow,
    CrossRecordLabel,
}

pub(in crate::domain_computation::primary_graph) fn retained_status_program(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    breadth: RetentionMutationBreadth,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let other = matches!(breadth, RetentionMutationBreadth::CrossRecordLabel)
        .then(|| resolved_account(world, "unrelated", request));
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
            match breadth {
                RetentionMutationBreadth::Narrow => {}
                RetentionMutationBreadth::CrossRecordLabel => {
                    let other = reader
                        .resolve_entity(AccountStatus::reference(), "unrelated".to_owned())
                        .unwrap();
                    reader
                        .require_decision_field(&other, AccountLabel::reference())
                        .unwrap();
                }
            }
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(account).unwrap();
    effects
        .write_field(&account, AccountStatus::reference(), replacement.to_owned())
        .unwrap();
    if let Some(other) = other {
        let other = effects.existing_entity(&other).unwrap();
        effects
            .write_field(
                &other,
                AccountLabel::reference(),
                "retention-breadth".to_owned(),
            )
            .unwrap();
    }
    effects
        .emit_external(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(format!("status-retention:{replacement}")),
        )
        .unwrap();
    effects.finish().unwrap()
}
