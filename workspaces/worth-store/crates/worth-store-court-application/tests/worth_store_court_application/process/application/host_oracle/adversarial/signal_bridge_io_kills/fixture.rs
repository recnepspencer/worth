//! Shared Signal + Bridge + World fixture, observation, and layout-stabilization
//! helpers for C4b.1 and C4b.3 adversarial suites.
use super::super::super::graceful::signal::operations::{
    product_generations, signal_oracle_details, SignalOracleDetails,
};
use super::super::build_id;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RecordedProductBranchReference;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::{
        WorthQueryApplicationCommitOutcome as CommitOutcome, WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationIdempotencyBinding, WorthQueryConditionalDefinitionPublicationOutcome,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
    product::WorthQueryProductBranch,
    runtime::RuntimeWorldCancellationSource,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::{
    contribution_installation::*, schema::AmendTemporal,
    world::program_amendment::commit_definition_on_program_lane,
};
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

pub(in super::super) type SignalHome =
    WorthQueryApplicationHome<TemporalHostSchema, TemporalInstallationProgram>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct StableNativeBasis {
    pub(in super::super) branch_id: worth_relational::facade::history::BranchId,
    pub(in super::super) truth_version: worth_relational::facade::branch::RelationalBranchVersion,
    pub(in super::super) schema_commitment: [u8; 32],
    pub(in super::super) posture: worth_relational::facade::branch::RelationalBranchBasisPosture,
}

#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct SignalSnapshot {
    pub(in super::super) intent: (String, u64, String, String),
    pub(in super::super) world_commit_ordinal: u64,
    pub(in super::super) details: SignalOracleDetails,
    pub(in super::super) native_basis: StableNativeBasis,
}

pub(in super::super) struct SignalOpen<'a> {
    pub(in super::super) location: Location,
    pub(in super::super) build: &'a str,
    pub(in super::super) initialized: Arc<AtomicUsize>,
    pub(in super::super) contacts: ContactCounters,
    pub(in super::super) placement: Option<AdversarialPlacement>,
}

pub(in super::super) fn open_signal_home(
    cfg: SignalOpen<'_>,
    execution: &ExecutionRequest<'_, '_>,
) -> SignalHome {
    let declaration = TemporalHostSchema::declaration().unwrap();
    let (installation_predicate, _) = Predicate::controlled(cfg.contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(cfg.contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts: cfg.contacts,
        install_route: true,
        definition_only: true,
        multi_intent_targets: false,
    };
    let init = cfg.initialized;
    let request = WorthQueryApplicationHomeRequest::new(
        cfg.location,
        ApplicationProgramAuthoring::<TemporalHostSchema, TemporalInstallationProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (configuration,),
        build_id(cfg.build),
        publication_limit::limits(),
        move |_, graph, installed| {
            init.fetch_add(1, Ordering::SeqCst);
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    );
    let request = match cfg.placement {
        Some(p) => host_observation::opening::attach_placement(request, p).0,
        None => request,
    };
    (*with_bootstrap_advancement_from_request(*execution, |phase| {
        Box::new(request.open_in_advancement(&phase))
    })
    .unwrap())
    .unwrap_or_else(|e| panic!("Signal Home open: {e:?}"))
}

pub(in super::super) fn commit_mixed(
    home: &SignalHome,
    branch: WorthQueryProductBranch,
    ordinal: u8,
) -> CommitOutcome {
    let app = home.runtime();
    let cond = app.conditional::<TemporalConditional>().unwrap();
    let installed = cond.as_ref().as_ref().unwrap();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    commit_definition_on_program_lane(
        app,
        &installed.clock,
        &installed.invariant,
        selected,
        &format!("adv-mixed-{ordinal}"),
        Arc::new(provider),
        ordinal,
    )
}

fn publish_definition_outcome(
    home: &SignalHome,
    branch: WorthQueryProductBranch,
) -> WorthQueryConditionalDefinitionPublicationOutcome {
    let app = home.runtime();
    let cond = app.conditional::<TemporalConditional>().unwrap();
    let installed = cond.as_ref().as_ref().unwrap();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    selected
        .publish_conditional_definition(
            &installed.clock,
            Arc::new(provider),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
}

pub(in super::super) fn publish_definition(home: &SignalHome, branch: WorthQueryProductBranch) {
    assert!(matches!(
        publish_definition_outcome(home, branch),
        WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
    ));
}

pub(in super::super) fn observe_signal(home: &SignalHome) -> SignalSnapshot {
    let app = home.runtime();
    let branch = app.current_world();
    let row = publication_limit::read_intent_on_branch(app, branch);
    let details = signal_oracle_details(home, branch);
    let (world_commit_ordinal, native_basis) = app
        .with_host_advancement(|phase| {
            let selected = app.on_branch(branch).select(&phase).unwrap();
            let prod = selected.product();
            let desc = prod.relational_basis_descriptor();
            (
                prod.selected_commit().ordinal(),
                StableNativeBasis {
                    branch_id: desc.branch_id().clone(),
                    truth_version: desc.truth_version(),
                    schema_commitment: desc.schema_commitment(),
                    posture: desc.posture(),
                },
            )
        })
        .unwrap();
    SignalSnapshot {
        intent: (row.identity, row.revision, row.lifecycle, row.input),
        world_commit_ordinal,
        details,
        native_basis,
    }
}

/// Observe and admit the recovered Bridge conditional definition on `branch`
/// BEFORE publishing any successor definition, checking that the recovered
/// Bridge lowering is live on the recovered Signal basis (`admitted.check()`),
/// that the conditional runtime holds the exact single installed binding/clock/
/// graph, and that zero provider contacts occur. Returns the binding digest.
pub(in super::super) fn verify_recovered_bridge_definition(
    home: &SignalHome,
    branch: WorthQueryProductBranch,
) -> [u8; 32] {
    let app = home.runtime();
    let insp = app.runtime().inspect_conditional_runtime();
    assert_eq!(
        (
            insp.installed_binding_count(),
            insp.managed_clock_count(),
            insp.signal_graph_count(),
        ),
        (0, 0, 1),
        "recovered Bridge conditional runtime must match MemoryOnly (definition_only: 1 live signal graph)"
    );
    let cond = app.conditional::<TemporalConditional>().unwrap();
    let installed = cond.as_ref().as_ref().unwrap();
    let before_contacts = installed.contacts.snapshot();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    let admitted = selected
        .admit_application_conditional_definition(&installed.clock, Arc::new(provider))
        .expect("recovered Bridge conditional definition must admit on recovered Signal basis");
    let def_id = *admitted.identity();
    let _checked = admitted
        .check()
        .expect("recovered Bridge lowering must be served on recovered Signal basis");
    assert_eq!(
        installed.contacts.snapshot(),
        before_contacts,
        "checking recovered Bridge def never calls providers"
    );
    def_id
}

pub(in super::super) fn assert_bridge_publication_healthy(
    home: &SignalHome,
    branch: WorthQueryProductBranch,
    before_ord: u64,
    before_sig_gen: u64,
) {
    use WorthQueryConditionalDefinitionPublicationOutcome::Performed;
    verify_recovered_bridge_definition(home, branch);
    let Performed(p1) = publish_definition_outcome(home, branch) else {
        panic!("Bridge def must publish")
    };
    assert!(
        p1.definition_generation() >= 2,
        "Bridge def gen must advance past reopened baseline"
    );
    assert_eq!(
        p1.product_commit().ordinal(),
        before_ord + 1,
        "Bridge publication advances World +1"
    );
    assert_eq!(
        product_generations(home, branch).0,
        before_sig_gen + 1,
        "Bridge pub advances signal_generation +1"
    );
    let Performed(p2) = publish_definition_outcome(home, branch) else {
        panic!("repeat Bridge def must publish")
    };
    assert_eq!(
        p2.definition_generation(),
        p1.definition_generation() + 1,
        "Bridge def gen advances +1"
    );
    assert_eq!(
        p2.product_commit().ordinal(),
        p1.product_commit().ordinal() + 1,
        "repeat Bridge advances +1"
    );
    assert_eq!(
        product_generations(home, branch).0,
        before_sig_gen + 2,
        "repeat Bridge advances signal_generation +2"
    );
}

/// Compute the admitted idempotency key directly at admission (independent of
/// any live or recovered receipt) for `commit_mixed(..., ordinal)`.
pub(in super::super) fn admitted_signal_key_identity(ordinal: u8) -> [u8; 32] {
    *WorthQueryApplicationIdempotencyBinding::for_host_commit::<
        TemporalHostSchema,
        AmendTemporal,
        _,
        _,
    >(
        &("program-definition-amendment", ordinal),
        &("program-definition-amendment", ordinal),
    )
    .unwrap()
    .key_identity()
}

pub(in super::super) fn assert_live_receipt_bound(
    rec: &WorthQueryApplicationCommitReceipt,
    snap: &SignalSnapshot,
    expected_after: &SignalSnapshot,
    expected_key: [u8; 32],
    label: &str,
) {
    let p = rec.committed_product_publication();
    assert_eq!(
        *rec.idempotency_binding().key_identity(),
        expected_key,
        "{label}: live key"
    );
    assert_eq!(
        Some(p.publication_attempt().ordinal()),
        snap.details.head_attempt_ordinal,
        "{label}: live attempt"
    );
    assert_eq!(
        Some(rec.commit_reference().commit_id.0),
        snap.details.head_native_commit_id,
        "{label}: live commit"
    );
    assert_eq!(
        Some(rec.commit_reference().version_id.as_u64()),
        snap.details.head_native_version_id,
        "{label}: live version"
    );
    assert_eq!(
        p.composite_commit().ordinal(),
        snap.world_commit_ordinal,
        "{label}: live composite"
    );
    assert_eq!(
        p.product_generation().get(),
        snap.details.world_reference_generation,
        "{label}: live prod_gen"
    );
    assert_eq!(
        rec.basis_descriptor().truth_version(),
        expected_after.native_basis.truth_version,
        "{label}: live truth_version"
    );
}

pub(in super::super) fn assert_recovered_receipt_bound(
    rec: &WorthQueryRecoveredApplicationCommitReceipt,
    snap: &SignalSnapshot,
    expected_after: &SignalSnapshot,
    expected_key: [u8; 32],
    label: &str,
) {
    let (generation, commit) = match rec.new_product_head() {
        RecordedProductBranchReference::Home(h) => {
            (h.generation().get(), h.selected_commit().ordinal())
        }
        RecordedProductBranchReference::Memory(h) => (h.generation(), h.selected_commit()),
    };
    assert_eq!(
        *rec.idempotency_key_identity(),
        expected_key,
        "{label}: recovered key"
    );
    assert_eq!(
        Some(rec.attempt_identity().ordinal()),
        snap.details.head_attempt_ordinal,
        "{label}: recovered attempt"
    );
    assert_eq!(
        Some(rec.commit_reference().commit_id.0),
        snap.details.head_native_commit_id,
        "{label}: recovered commit"
    );
    assert_eq!(
        Some(rec.commit_reference().version_id.as_u64()),
        snap.details.head_native_version_id,
        "{label}: recovered version"
    );
    assert_eq!(
        commit, snap.world_commit_ordinal,
        "{label}: recovered composite"
    );
    assert_eq!(
        generation, snap.details.world_reference_generation,
        "{label}: recovered prod_gen"
    );
    assert_eq!(
        rec.basis_descriptor().truth_version(),
        expected_after.native_basis.truth_version,
        "{label}: recovered truth_version"
    );
}
