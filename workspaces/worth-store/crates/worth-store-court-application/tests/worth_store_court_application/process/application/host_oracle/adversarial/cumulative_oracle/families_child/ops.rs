//! Domain operations, receipt identity projection, & inspection helpers for C4b.4 families.
use super::super::super::build_id;
use super::super::super::correction_fixture::{scope as corr_scope, LabelHome};
use std::collections::BTreeMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::Duration;
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationCorrectionDenial as CorrDenial,
        WorthQueryApplicationCorrectionPreparation as Prepared,
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryBranchAdoptionPublicationOutcome, WorthQueryCorrectionHistoryLimits,
    },
    application_installation::WorthQueryProgramOwner,
    primary_graph::{
        QueryCorrectionMaterialDenial, WorthQueryApplicationCommitReceipt,
        WorthQueryCorrectionDerivationDenial, WorthQueryCorrectionSourceCoordinates,
        WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
        WorthQueryExternalTransportOutcome as TransportOutcome,
        WorthQueryRecordedCorrectionSource, WorthQueryRecoveredApplicationCommitReceipt,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_operator, request_scope},
    programs::{RetentionProgramP0, RetentionProgramP1},
    schema::DocumentRetentionSchema as Schema,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture as label;
use worth_query_testkit_host::facade::host::{
    application_runtime_surface as identity,
    dispatch_home::{
        self as dispatch, notify::NotifyAccountBalance, schema::DispatchHomeSchema,
        DispatchHomeProgram,
    },
};
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement,
    host_observation::{self, projection::HeadProjection},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(in super::super) struct ReceiptId {
    pub(in super::super) commit_id: u64,
    pub(in super::super) version_id: u64,
    pub(in super::super) world_incarnation: u64,
    pub(in super::super) world_generation: u64,
    pub(in super::super) world_commit: u64,
}

impl ReceiptId {
    pub(in super::super) fn from_live(r: &WorthQueryApplicationCommitReceipt) -> Self {
        let p = r.committed_product_publication();
        Self {
            commit_id: r.commit_reference().commit_id.0,
            version_id: r.commit_reference().version_id.as_u64(),
            world_incarnation: p.product_incarnation().ordinal(),
            world_generation: p.product_generation().get(),
            world_commit: p.composite_commit().ordinal(),
        }
    }

    pub(in super::super) fn from_recovered(
        r: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> Self {
        let h = HeadProjection::observe(r.new_product_head());
        Self {
            commit_id: r.commit_reference().commit_id.0,
            version_id: r.commit_reference().version_id.as_u64(),
            world_incarnation: h.incarnation,
            world_generation: h.generation,
            world_commit: h.commit,
        }
    }
}

pub(super) enum ReverseFate {
    Committed(ReceiptId),
    RecoveredCommitted(ReceiptId),
    AlreadyCommitted(ReceiptId),
    ProductUnpublished,
    PreparationDischarged,
    SourceAlreadyReversed,
}

pub(in super::super) fn cadence_map<S, P>(
    home: &WorthQueryApplicationHome<S, P>,
    exec: &ExecutionRequest<'_, '_>,
) -> BTreeMap<String, (u64, u64)> {
    home.inspect_checkpoint(exec)
        .unwrap()
        .journals
        .into_iter()
        .map(|j| (j.name.to_owned(), (j.covered, j.head)))
        .collect()
}

pub(in super::super) fn custody_count<S, P>(home: &WorthQueryApplicationHome<S, P>) -> usize
where
    S: worth_query_decl::facade::application_schema::ApplicationSchema,
{
    home.runtime()
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap()
        .rows()
        .len()
}

pub(in super::super) fn adopt_p1(home: &WorthQueryApplicationHome<Schema, RetentionProgramP0>) {
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let progs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs();
    let req = progs.compare(&target).unwrap();
    let prep = progs
        .adopt(&req)
        .unwrap()
        .workflow(
            worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::retire_and_cancel_unperformed(),
        )
        .prepare(1_024)
        .unwrap();
    assert!(matches!(
        prep.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

pub(in super::super) fn observe_adoption_rev(
    home: &WorthQueryApplicationHome<Schema, RetentionProgramP0>,
) -> worth_query_decl::facade::application_program::ApplicationProgramRevision {
    home.runtime()
        .with_host_advancement(|p| {
            *home
                .runtime()
                .runtime()
                .on_branch(home.runtime().current_world())
                .select(&p)
                .unwrap()
                .inspect_selected_program()
                .unwrap()
                .revision()
        })
        .unwrap()
}

pub(super) fn settle_custody_if_any<S, P>(
    home: &WorthQueryApplicationHome<S, P>,
    exec: &ExecutionRequest<'_, '_>,
) where
    S: worth_query_decl::facade::application_schema::ApplicationSchema,
{
    let rows = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap();
    for row in rows.rows() {
        let rec = home
            .runtime()
            .runtime()
            .readmit_product_publication_recovery(row.handle())
            .unwrap();
        loop {
            match rec.continue_original_in_request(exec).unwrap() {
                WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                WorldRecoveredContinuationOutcome::Published { .. } => break,
                other => panic!("custody continuation failed: {other:?}"),
            }
        }
    }
}

struct FileLog {
    path: Option<PathBuf>,
}

impl WorthQueryExternalEffectTransport for FileLog {
    fn dispatch(&self, _: WorthQueryExternalDispatchRequest<'_>) -> TransportOutcome {
        if let Some(p) = &self.path {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .unwrap();
            writeln!(f, "dispatch").unwrap();
            f.sync_data().unwrap();
        }
        TransportOutcome::Completed
    }
}

pub(super) fn dispatch_count(p: &Path) -> usize {
    std::fs::read_to_string(p).map_or(0, |s| s.lines().filter(|l| !l.is_empty()).count())
}

pub(super) fn open_dispatch(
    loc: Location,
    build: &str,
    seed: bool,
    placement: Option<AdversarialPlacement>,
    log: Option<PathBuf>,
    exec: &ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationHome<DispatchHomeSchema, DispatchHomeProgram> {
    let req =
        dispatch::request_at_location(loc, build_id(build), Arc::new(AtomicUsize::new(0)), seed);
    let req = match placement {
        Some(p) => host_observation::opening::attach_placement(req, p).0,
        None => req,
    };
    let h = with_bootstrap_advancement_from_request(*exec, |p| req.open_in_advancement(&p))
        .unwrap()
        .unwrap();
    h.runtime()
        .runtime()
        .install_external_effect_transport(Arc::new(FileLog { path: log }))
        .unwrap();
    h
}

pub(super) fn notify_dispatch(
    home: &WorthQueryApplicationHome<DispatchHomeSchema, DispatchHomeProgram>,
    key: u64,
    bal: u64,
) -> Outcome<(), u64> {
    let sc = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    );
    let p = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &sc,
    );
    home.runtime()
        .request(&p, &sc)
        .advanced()
        .mutate(NotifyAccountBalance { balance: bal })
        .without_source()
        .idempotency(&key)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

pub(super) fn corr_summary(home: &LabelHome) -> (String, String) {
    let sc = corr_scope();
    let rt = home.runtime();
    let p = label::authentication::authenticate_external(
        rt.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &sc,
    );
    let rows = rt
        .request(&p, &sc)
        .query(label::AccountSummaryParameters)
        .execute()
        .unwrap();
    let r = &rows.rows()[0];
    (r.status.clone(), r.label.clone())
}

pub(super) fn seed_label_edit(home: &LabelHome) -> WorthQueryCorrectionSourceCoordinates {
    let sc = corr_scope();
    let rt = home.runtime();
    let p = label::authentication::authenticate_external(
        rt.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &sc,
    );
    let Outcome::Committed { receipt, .. } = rt
        .request(&p, &sc)
        .advanced()
        .mutate(label::correction_source::label_edit::LabelEdit {
            status: "open".into(),
            label: "changed".into(),
        })
        .without_source()
        .idempotency(&"orig".to_owned())
        .execute_in_program(rt, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("seed edit");
    };
    receipt.correction_source().coordinates()
}

pub(super) fn reverse_attempt(
    home: &LabelHome,
    coords: &WorthQueryCorrectionSourceCoordinates,
    key: &str,
) -> ReverseFate {
    let sc = corr_scope();
    let rt = home.runtime();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(8).unwrap());
    let p = label::authentication::authenticate_external(
        rt.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &sc,
    );
    let src = rt
        .request(&p, &sc)
        .corrections()
        .select_coordinates(coords, limits)
        .unwrap();
    let fate = match rt
        .request(&p, &sc)
        .corrections()
        .reverse(&src)
        .idempotency(key)
        .limits(limits)
        .prepare()
    {
        Ok(Prepared::Prepared(prep)) => match prep.publish().unwrap() {
            Outcome::Committed { receipt, .. } => {
                ReverseFate::Committed(ReceiptId::from_live(&receipt))
            }
            Outcome::RecoveredCommitted(r) => {
                ReverseFate::RecoveredCommitted(ReceiptId::from_recovered(&r))
            }
            Outcome::AlreadyCommitted(r) => ReverseFate::AlreadyCommitted(ReceiptId::from_live(&r)),
            other => panic!("reverse publish unexpected: {other:?}"),
        },
        Ok(Prepared::Settled(Outcome::AlreadyCommitted(r))) => {
            ReverseFate::AlreadyCommitted(ReceiptId::from_live(&r))
        }
        Ok(Prepared::Settled(Outcome::RecoveredCommitted(r))) => {
            ReverseFate::RecoveredCommitted(ReceiptId::from_recovered(&r))
        }
        Ok(Prepared::ProductUnpublished(_))
        | Ok(Prepared::Settled(Outcome::Commit(
            worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(_),
        ))) => ReverseFate::ProductUnpublished,
        Err(CorrDenial::Derivation(WorthQueryCorrectionDerivationDenial::Material(
            QueryCorrectionMaterialDenial::PreparationDischarged,
        ))) => ReverseFate::PreparationDischarged,
        Err(CorrDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceAlreadyReversed,
        )) => ReverseFate::SourceAlreadyReversed,
        Ok(_) => panic!("reverse prepare returned unexpected variant"),
        Err(e) => panic!("reverse prepare failed: {e:?}"),
    };
    drop(src);
    fate
}
