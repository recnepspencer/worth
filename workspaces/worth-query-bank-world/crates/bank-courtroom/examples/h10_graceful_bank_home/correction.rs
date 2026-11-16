//! Same installed Bank Home: native fork, independent reversal occurrences and exact cold reapply.
use super::installation::Home;
use bank_domain::schema::{AccountStatus, FreezeEstateAccount, RestoreRecordedEstateFreeze};
use bank_server::{
    BankAssessedPaymentsApplication as BankApplication, BankAuthenticatedPrincipal,
    BankHomeRuntime, EstateAccountFreezeAction, RestoreFrozenAccountAction,
};
use worth_query_bank_testkit_server::facade::{
    estate_operations::freeze_account::fixture::{ESTATE, ESTATE_ACCOUNT},
    support::request_scope,
};
use worth_query_host::facade::product::{
    WorthQueryProductBranchCreateError as Fork,
    WorthQueryProductBranchCreationContinuation as Continued,
    WorthQueryProductBranchRecoveryDenial,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationCorrectionDenial as Entry, WorthQueryCorrectionHistoryLimits,
        WorthQueryOperationCorrectionPreparation as Preparation, WorthQuerySourceCompletion,
        WorthQuerySourceOnlyAdvance as Advance,
    },
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryCorrectionDerivationDenial as Derivation,
        WorthQueryCorrectionSourceCoordinates,
    },
    product::WorthQueryProductBranch,
};

/// Only descriptive original/reversal coordinates cross the graceful handoff.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reversals {
    original: WorthQueryCorrectionSourceCoordinates,
    root: WorthQueryCorrectionSourceCoordinates,
    child: WorthQueryCorrectionSourceCoordinates,
}
pub(super) fn reverse_on_both(home: &Home, principal: &BankAuthenticatedPrincipal) -> Reversals {
    let root = home.runtime().runtime().current_world();
    let original = freeze(home, principal);
    assert_eq!(status(home, principal, root), AccountStatus::Frozen);
    let child = fork(home);
    assert_eq!(status(home, principal, child), AccountStatus::Frozen);
    let root_reversal = reverse(home, principal, root, &original, "h10-reverse-root");
    let child_reversal = reverse(home, principal, child, &original, "h10-reverse-child");
    assert_ne!(
        root_reversal.commit_reference(),
        child_reversal.commit_reference()
    );
    for receipt in [&root_reversal, &child_reversal] {
        assert_eq!(
            receipt
                .aftermath_causality()
                .unwrap()
                .correction_lineage()
                .root()
                .coordinates(),
            original
        );
    }
    assert_eq!(status(home, principal, root), AccountStatus::Open);
    assert_eq!(status(home, principal, child), AccountStatus::Open);
    Reversals {
        original,
        root: root_reversal.correction_source().coordinates(),
        child: child_reversal.correction_source().coordinates(),
    }
}

pub(super) fn reapply_recovered_child(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    reversals: Reversals,
) {
    let Reversals {
        original,
        root: root_reversal,
        child: child_reversal,
    } = reversals;
    let (root, child) = fresh_branches(home);
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let root_request = bank.request(&principal, &scope).on_branch(root);
    let child_request = bank.request(&principal, &scope).on_branch(child);
    let root_source = root_request
        .corrections()
        .select_coordinates(&original, limits())
        .unwrap();
    let child_source = child_request
        .corrections()
        .select_coordinates(&original, limits())
        .unwrap();
    let root_page = root_request
        .corrections()
        .alternatives(&root_source, limits(), page_size())
        .unwrap();
    let child_page = child_request
        .corrections()
        .alternatives(&child_source, limits(), page_size())
        .unwrap();
    assert!(root_page.continuation().is_none());
    assert!(child_page.continuation().is_none());
    assert_eq!(root_page.entries().len(), 1);
    assert_eq!(child_page.entries().len(), 1);
    let root_entry = root_page.entries()[0].clone();
    let child_entry = child_page.entries()[0].clone();
    assert_eq!(root_entry.performed_commit(), root_reversal.relational());
    assert_eq!(child_entry.performed_commit(), child_reversal.relational());
    assert_ne!(root_entry, child_entry);
    let runtime = home.runtime().runtime();
    let publication = |branch| {
        runtime
            .with_application_advancement(&scope, |phase| {
                runtime
                    .on_branch(branch)
                    .select(&phase)
                    .unwrap()
                    .product()
                    .selected_commit()
                    .clone()
            })
            .unwrap()
    };
    let before = (publication(root), publication(child));
    let consumption_before = (
        root_page.consumption_entries_examined(),
        child_page.consumption_entries_examined(),
    );
    let denial = child_page
        .select(&root_entry, limits())
        .err()
        .expect("root reversal is foreign to child page");
    assert!(
        matches!(denial, Entry::Derivation(Derivation::ForeignAlternative)),
        "{denial:?}"
    );
    assert_eq!((publication(root), publication(child)), before);
    let root_after = root_request
        .corrections()
        .alternatives(&root_source, limits(), page_size())
        .unwrap();
    let child_after = child_request
        .corrections()
        .alternatives(&child_source, limits(), page_size())
        .unwrap();
    assert!(root_after.continuation().is_none() && child_after.continuation().is_none());
    assert_eq!(root_after.entries(), root_page.entries());
    assert_eq!(child_after.entries(), child_page.entries());
    assert_eq!(
        (
            root_after.consumption_entries_examined(),
            child_after.consumption_entries_examined()
        ),
        consumption_before
    );

    let alternative = child_page.select(&child_entry, limits()).unwrap();
    let Preparation::Prepared(prepared) = child_request.corrections().reapply(&alternative)
            .idempotency("reapply-freeze-child").limits(limits())
            .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, EstateAccountFreezeAction>(home.runtime()).unwrap()
        else { panic!("the cold child inverse's declared empty output contract admits reapplication"); };
    let Advance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("child reapplication must complete through its original source owner");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("first use must publish a fresh child occurrence");
    };
    let receipt = receipt.clone();
    let lineage = receipt.aftermath_causality().unwrap().correction_lineage();
    assert_eq!(lineage.root().coordinates(), original);
    assert_eq!(
        lineage.selected_alternative().unwrap().relational(),
        child_reversal.relational()
    );
    assert_eq!(
        lineage.active_reversal().unwrap().relational(),
        child_reversal.relational()
    );
    assert!(completed.close(&child_request).is_ok());
    let child_after_reapply = publication(child);
    let Preparation::Continued(Advance::Completed(replayed)) = child_request.corrections().reapply(&alternative)
            .idempotency("reapply-freeze-child").limits(limits())
            .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, EstateAccountFreezeAction>(home.runtime()).unwrap()
        else { panic!("same-key replay must recover the performed child occurrence"); };
    let WorthQuerySourceCompletion::Recorded {
        receipt: recovered, ..
    } = replayed.evidence()
    else {
        panic!("same-key replay preserves recorded provenance");
    };
    assert_eq!(recovered.commit_reference(), receipt.commit_reference());
    assert!(replayed.close(&child_request).is_ok());
    assert_eq!(
        publication(child),
        child_after_reapply,
        "replay cannot republish"
    );
    assert_eq!(
        publication(root),
        before.0,
        "child publication cannot move root"
    );
    assert_eq!(status(home, &principal, child), AccountStatus::Frozen);
    assert_eq!(status(home, &principal, root), AccountStatus::Open);

    // Successful fresh preparation checks the root's active-reversal consumption
    // at its own exact basis. Do not publish it: final root truth remains Open.
    let root_page = root_request
        .corrections()
        .alternatives(&root_source, limits(), page_size())
        .unwrap();
    assert!(root_page.continuation().is_none());
    let root_alternative = root_page.select(&root_entry, limits()).unwrap();
    let Preparation::Prepared(unperformed_root) = root_request.corrections().reapply(&root_alternative)
            .idempotency("unperformed-root-reapply").limits(limits())
            .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, EstateAccountFreezeAction>(home.runtime()).unwrap()
        else { panic!("child consumption must leave the root reversal available"); };
    drop(unperformed_root);
    assert_eq!(
        (publication(root), status(home, &principal, root)),
        (before.0, AccountStatus::Open)
    );
}

fn freeze(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
) -> WorthQueryCorrectionSourceCoordinates {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let operation = request
        .mutate(FreezeEstateAccount::new(ESTATE, ESTATE_ACCOUNT))
        .without_source()
        .idempotency_owned(
            bank_domain::proposals::BankIdempotencyKey::new("h10-original-freeze").unwrap(),
        )
        .unwrap_or_else(|(_, _, cause)| panic!("freeze capture: {cause:?}"))
        .retain_durable_operation::<BankApplication, EstateAccountFreezeAction>(
            &request,
            home.runtime(),
        )
        .unwrap_or_else(|(_, cause)| panic!("durable freeze description: {cause:?}"));
    let Advance::Completed(completed) = operation.advance(&request) else {
        panic!("the real freeze must complete");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("the real freeze must be fresh");
    };
    let original = receipt.correction_source().coordinates();
    assert!(completed.close(&request).is_ok());
    original
}

fn reverse(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    branch: WorthQueryProductBranch,
    original: &WorthQueryCorrectionSourceCoordinates,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope).on_branch(branch);
    let source = request
        .corrections()
        .select_coordinates(original, limits())
        .unwrap();
    let Preparation::Prepared(prepared) = request.corrections().reverse(&source).idempotency(key).limits(limits())
        .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, RestoreFrozenAccountAction>(home.runtime()).unwrap()
    else { panic!("the recorded freeze inverse must prepare on its exact target"); };
    let Advance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("the recorded freeze inverse must publish");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("each independent reversal must be fresh");
    };
    let receipt = receipt.clone();
    assert!(completed.close(&request).is_ok());
    receipt
}

fn status(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    branch: WorthQueryProductBranch,
) -> AccountStatus {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    bank.request(principal, &request_scope())
        .on_branch(branch)
        .query(bank_domain::queries::estate_case(ESTATE))
        .execute()
        .unwrap()
        .rows()[0]
        .account()
        .status()
}
fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap())
}
fn page_size() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(8).unwrap()
}

pub(super) fn fork(home: &Home) -> WorthQueryProductBranch {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let created = runtime
        .with_application_advancement(&scope, |phase| {
            runtime
                .branches()
                .fork(runtime.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    let mut original = match created {
        Ok(child) => return child,
        Err(Fork::ProductUnpublished(original)) => original,
        Err(cause) => panic!("freeze fork: {cause:?}"),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "original fork must settle"
        );
        match runtime
            .with_application_advancement(&scope, |phase| original.continue_in_advancement(&phase))
            .unwrap()
        {
            Ok(Continued::Performed(child)) => return child,
            Ok(Continued::NoEffect(cause)) => panic!("fork no effect: {cause:?}"),
            Ok(Continued::ReleasedOriginal(cause)) => panic!("fork released: {cause:?}"),
            Err(failure) => {
                assert_eq!(
                    failure.denial(),
                    WorthQueryProductBranchRecoveryDenial::Busy
                );
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}

pub(super) fn fresh_branches(home: &Home) -> (WorthQueryProductBranch, WorthQueryProductBranch) {
    let runtime = home.runtime().runtime();
    let root = runtime.current_world();
    let inventory = runtime
        .with_application_advancement(&request_scope(), |phase| {
            runtime.branches().inventory(&phase)
        })
        .unwrap()
        .unwrap();
    assert_eq!(inventory.iter().len(), 2);
    let child = inventory.iter().find(|branch| *branch != root).unwrap();
    (root, child)
}
