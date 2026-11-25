//! Public #388: a Directory Home restores each correction witness from its
//! durable custody (the retained native effect joined to its published World
//! original) and that witness equals the one its session captured.
use super::numeric_home::*;
use super::stage_correction::{envelope, reverse, values, OtherAction, KEYS, SET};
use super::*;
use model::{
    authentication::{authenticate_operator, request_scope},
    correction::RecordedEdit,
    edit::{Edit, Value},
};
use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryEditEnvelopeOutcome},
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryCorrectionSourceCoordinates,
        WorthQueryCorrectionSourceMember,
    },
    WorthQueryAnyCorrectionCustodyOutcome, WorthQueryAnySourceCustody,
    WorthQueryAnySourceCustodyStep,
};
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};

/// Reapplies the reversal of `source` under `key` and returns its receipt.
macro_rules! reapply {
    ($home:expr, $key:expr, $source:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($home.primary_graph().installed_schema(), &scope);
        let request = $home.runtime().request(&principal, &scope);
        let selected = request.corrections().select($source, limits()).expect("select");
        let alternatives = request
            .corrections()
            .alternatives(&selected, limits(), std::num::NonZeroUsize::new(8).unwrap())
            .expect("alternatives");
        let alternative = alternatives.entries().first().expect("one alternative");
        let selected = alternatives.select(alternative, limits()).expect("alternative");
        let prepared = request
            .corrections()
            .reapply(&selected)
            .idempotency($key)
            .limits(limits())
            .prepare_operation::<RecordedEdit, DirectoryProgram, model::program::Action>($home.runtime())
            .unwrap_or_else(|denial| panic!("reapply prepares: {denial:?}"));
        match WorthQueryAnySourceCustody::accept_correction(prepared, &request) {
            WorthQueryAnyCorrectionCustodyOutcome::Custody(WorthQueryAnySourceCustodyStep::Committed { evidence, .. }) => {
                evidence.live_receipt().expect("a live reapply receipt").clone()
            }
            _ => panic!("reapply commits"),
        }
    }};
}

/// Every witness kind on one warm Directory Home: the envelope commit (plain),
/// reversals of the Primary stage and both authored stages, a reapplication of
/// stage 0 and the reversal of that reapplication, which carries its prior.
fn corrected(execution: worth_execution::ExecutionRequest<'_, '_>) -> NumericHome {
    let home = open_program_home_with(fixture::location(), execution, true, STAGE_NUMBERS);
    let receipts = envelope!(home);
    for (stage, key) in [(0, "undo-0"), (1, "undo-1"), (2, "undo-2")] {
        reverse!(home, DirectoryProgram, key, |corrections| corrections.select(&receipts[stage], limits()))
            .unwrap_or_else(|cause| panic!("stage {stage} reverses: {cause}"));
    }
    let redo = reapply!(home, "redo-0", &receipts[0]);
    reverse!(home, DirectoryProgram, "undo-redo-0", |corrections| corrections.select(&redo, limits()))
        .unwrap_or_else(|cause| panic!("the reapplication reverses: {cause}"));
    assert_eq!(values(home.runtime()), [5, 5, 5]);
    home
}

/// The shape of a witness's sealed fact, or `plain` when it has none.
fn shape(aftermath: &Option<worth_query_execution::facade::primary_graph::WorthQueryCommittedAftermathCausality>) -> String {
    let Some(fact) = aftermath else { return "plain".to_owned() };
    let member = match fact.recorded_source().member() {
        WorthQueryCorrectionSourceMember::Primary => "primary",
        WorthQueryCorrectionSourceMember::AuthoredStage { .. } => "stage",
    };
    let prior = if fact.correction_lineage().prior_transition().is_some() { " with prior" } else { "" };
    format!("{:?} {member}{prior}", fact.role())
}

#[test]
fn a_directory_home_restores_every_witness_its_session_captured() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = corrected(execution);
    let core = home.primary_graph().runtime_for_test();
    let provider = core.primary_provider_for_test();
    let (originals, owner) = core.durable_originals_for_test(&execution).expect("a durable Home");
    let mut shapes = std::collections::BTreeSet::new();
    for receipt in provider.completed_receipts_for_test() {
        let session = provider.capture_correction_witness_for_test(execution, &receipt).unwrap().expect("session");
        let restored = provider
            .restore_correction_witness(execution, &originals, owner, &receipt)
            .unwrap_or_else(|denial| panic!("{receipt:?} restores: {denial:?}"))
            .unwrap_or_else(|| panic!("{receipt:?} has a published original"));
        assert_eq!(restored.world, session.world, "{receipt:?} World commit");
        assert_eq!(restored.aftermath, session.aftermath, "{receipt:?} sealed fact");
        shapes.insert(shape(&session.aftermath));
    }
    let expected = ["plain", "RecordedReverse primary", "RecordedReverse stage",
        "RecordedReapply primary with prior", "RecordedReverse primary with prior"];
    for kind in expected {
        assert!(shapes.contains(kind), "{kind} is covered: {shapes:?}");
    }
}

/// A receipt naming no retained native effect restores nothing, never a plain witness.
#[test]
fn a_directory_home_restores_no_witness_for_a_receipt_without_an_effect() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home_with(fixture::location(), execution, true, STAGE_NUMBERS);
    let real = envelope!(home)[0].commit_reference().clone();
    let core = home.primary_graph().runtime_for_test();
    let provider = core.primary_provider_for_test();
    let (originals, owner) = core.durable_originals_for_test(&execution).expect("a durable Home");
    assert!(provider.restore_correction_witness(execution, &originals, owner, &real).unwrap().is_some());
    let unknown = RelationalCommitReceipt { commit_id: CommitId(u64::MAX), ..real.clone() };
    let other_version = RelationalCommitReceipt {
        version_id: worth_relational::facade::identity::VersionId(real.version_id.0 + 1_000),
        ..real.clone()
    };
    for receipt in [unknown, other_version] {
        let restored = provider.restore_correction_witness(execution, &originals, owner, &receipt);
        assert!(matches!(restored, Ok(None)), "{receipt:?}: {:?}", restored.map(|w| w.map(|w| shape(&w.aftermath))));
    }
    assert_eq!(values(home.runtime()), SET);
}

/// A recorded coordinate naming another World ordinal or owner for the same
/// Relational commit seals nothing; a fact naming another child is inconsistent.
#[test]
fn a_directory_home_refuses_a_forged_coordinate_in_a_restored_fact() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = corrected(execution);
    let core = home.primary_graph().runtime_for_test();
    let provider = core.primary_provider_for_test();
    let (originals, owner) = core.durable_originals_for_test(&execution).expect("a durable Home");
    let with_prior = provider
        .completed_receipts_for_test()
        .into_iter()
        .find(|receipt| {
            let witness = provider.capture_correction_witness_for_test(execution, receipt).unwrap().unwrap();
            shape(&witness.aftermath) == "RecordedReverse primary with prior"
        })
        .expect("a reversal with its prior");
    let seal = |forge: &dyn Fn(&mut _)| {
        provider.seal_forged_durable_fact_for_test(execution, &originals, owner, &with_prior, |fact| forge(fact))
    };
    assert!(matches!(seal(&|_| {}), Ok(Some(_))), "the real fact seals");
    let ordinal = |c: &WorthQueryCorrectionSourceCoordinates| {
        WorthQueryCorrectionSourceCoordinates::new(
            c.world_owner(), c.world_commit() + 1, c.relational().clone(), c.member())
    };
    let memory = |c: &WorthQueryCorrectionSourceCoordinates| {
        WorthQueryCorrectionSourceCoordinates::new(
            worth_foundational::facade::WorldOwnerCoordinate::MemoryOnly { observation: 0 },
            c.world_commit(), c.relational().clone(), c.member())
    };
    for (label, forged) in [
        ("source ordinal", seal(&|fact| fact.source = ordinal(&fact.source))),
        ("root ordinal", seal(&|fact| fact.root = ordinal(&fact.root))),
        ("prior ordinal", seal(&|fact| fact.prior = fact.prior.as_ref().map(ordinal))),
        ("source owner", seal(&|fact| fact.source = memory(&fact.source))),
    ] {
        assert!(matches!(forged, Ok(None)), "{label}: {forged:?}");
    }
    let child = seal(&|fact| fact.child.commit_id = CommitId(fact.child.commit_id.0 + 1));
    assert!(matches!(child, Err(ref denial) if format!("{denial:?}") == "InconsistentCausality"), "{child:?}");
}
