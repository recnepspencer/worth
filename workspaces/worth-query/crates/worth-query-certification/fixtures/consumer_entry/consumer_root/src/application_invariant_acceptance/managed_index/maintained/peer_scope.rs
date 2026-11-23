//! Managed index reads made by the generated producer's own coupled decision:
//! its own scope (the applied candidate's aligned basis) and a peer scope that
//! shares the watch grant with it. A court arms each probe on its thread.
use super::{protected::access::grants::GrantResource, query, schema::*};
use std::cell::{Cell, RefCell};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::*,
};

/// What the generated producer's decision probes, when a court arms it.
#[derive(Clone, Copy, Debug)]
pub enum ProducerProbe {
    /// The producer's own Directory.
    Own,
    /// Another Directory, by identity.
    Peer(&'static str),
    OrderedOwn,
    Undeclared,
}

/// One probe's observed outcome: the selected identities, or its refusal.
pub type Observed = Result<Vec<String>, ProbeRefusal>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeRefusal {
    /// The typed managed index refusal.
    Refused(WorthQueryManagedIndexRefusal),
    /// The original request's cancellation, inside the read.
    Cancelled,
    /// The probed scope key names no live record at the read's basis.
    UnknownScope,
    /// Any other refusal.
    Undeclared,
    Other,
}

std::thread_local! {
    /// Seeds the `peer` Directory under the watch grant beside `catalog`.
    pub static SEED_PEERS: Cell<bool> = const { Cell::new(false) };
    pub static PRODUCER_PROBE: Cell<Option<ProducerProbe>> = const { Cell::new(None) };
    pub static OBSERVED: RefCell<Vec<Observed>> = const { RefCell::new(Vec::new()) };
    /// The generated source action also creates this Item in its Directory:
    /// an unpublished change the Parent's Source sees.
    pub static SOURCE_CREATES: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// The generated source action also creates this sibling Directory: a new
    /// scope entity in the same coupled envelope.
    pub static ORDERED_SUCCESSOR: Cell<Option<(&'static str, &'static str)>> = const { Cell::new(None) };
    pub static SOURCE_SIBLING: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// The producer probes through its partitioned computation's reader.
    pub static THROUGH_COMPUTATION: Cell<bool> = const { Cell::new(false) };
    /// Runs once after the next successful recorded read.
    #[allow(clippy::type_complexity)]
    pub static AFTER_READ: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
}

pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if !SEED_PEERS.get() {
        return Ok(());
    }
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Directory::reference(),
            WorthQueryApplicationEntityKey::new("peer").unwrap(),
        )
        .field(DirectoryIdentity::at("DirectoryIdentity"), "peer".to_owned())
        .field(DirectoryNote::at("DirectoryNote"), String::new()),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantResource::reference(),
        "watch-grant-peer",
        WorthQueryApplicationEntityKey::new("watch-access").unwrap(),
        WorthQueryApplicationEntityKey::new("peer").unwrap(),
    ))?;
    // `shelf` groups `catalog` and `shelved`, which shares no watch grant.
    for directory in ["shelf", "shelved"] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Directory::reference(),
                WorthQueryApplicationEntityKey::new(directory).unwrap(),
            )
            .field(DirectoryIdentity::at("DirectoryIdentity"), directory.to_owned())
            .field(DirectoryNote::at("DirectoryNote"), String::new()),
        )?;
    }
    for member in ["catalog", "shelved"] {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Shelves::reference(),
            &format!("shelf-{member}"),
            WorthQueryApplicationEntityKey::new("shelf").unwrap(),
            WorthQueryApplicationEntityKey::new(member).unwrap(),
        ))?;
    }
    Ok(())
}

/// The armed probe through the decision's own `index_query`; its outcome is
/// recorded and a refusal stops the decision with its typed cause. With
/// `THROUGH_COMPUTATION` the producer's partitioned computation probes instead.
pub fn probe<B>(
    reader: &mut DecisionReader<'_, '_, '_, Catalog, B>,
    own: &str,
) -> Result<(), HandlerExecutionDenial>
where
    B: worth_query_decl::facade::application_operation::ApplicationMutationBinding<Catalog>,
{
    let Some(probe) = PRODUCER_PROBE.get().filter(|_| !THROUGH_COMPUTATION.get()) else {
        return Ok(());
    };
    let rows = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
        directory: probed(probe, own),
        prefix: String::new(),
    });
    observe(rows).map(|_| ())
}

/// The Directory a probe names, from the operation's own.
pub fn probed(probe: ProducerProbe, own: &str) -> String {
    match probe {
        ProducerProbe::Own | ProducerProbe::OrderedOwn | ProducerProbe::Undeclared => own.to_owned(),
        ProducerProbe::Peer(peer) => peer.to_owned(),
    }
}

/// Records one decision read's outcome, runs the armed after-read hook on a
/// success, and returns the rows or the original typed refusal.
pub fn observe(
    rows: Result<Vec<query::ItemRow>, HandlerExecutionDenial>,
) -> Result<Vec<query::ItemRow>, HandlerExecutionDenial> {
    let (observed, result) = match rows {
        Ok(rows) => (
            Ok(rows.iter().map(|row| row.identity.clone()).collect()),
            Ok(rows),
        ),
        Err(cause) => match cause.downcast::<WorthQueryManagedIndexDenial>() {
            Ok(denial) => (
                Err(refusal(denial.cause())),
                Err(HandlerExecutionDenial::new(denial)),
            ),
            Err(cause) => (Err(ProbeRefusal::Other), Err(cause)),
        },
    };
    OBSERVED.with(|all| all.borrow_mut().push(observed));
    if result.is_ok()
        && let Some(hold) = AFTER_READ.with(|hold| hold.borrow_mut().take())
    {
        hold();
    }
    result
}

/// Classifies a managed index refusal for a court's comparison.
pub fn refusal(cause: WorthQueryManagedIndexDenialCause<'_>) -> ProbeRefusal {
    match cause {
        WorthQueryManagedIndexDenialCause::Refused(refusal) => ProbeRefusal::Refused(refusal),
        WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
            WorthQueryManagedIndexStop::Request(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::Cancelled,
            )),
        )) => ProbeRefusal::Cancelled,
        // The producer's coordinated source-query control observed the
        // original request's cancellation.
        WorthQueryManagedIndexDenialCause::CurrentControl(control)
            if matches!(
                control.kind(),
                WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            ) =>
        {
            ProbeRefusal::Cancelled
        }
        WorthQueryManagedIndexDenialCause::Scope(scope)
            if scope.kind()
                == WorthQueryEntityResolutionDenialKind::Refused(
                    WorthQueryEntityResolutionRefusalKind::UnknownEntity,
                ) =>
        {
            ProbeRefusal::UnknownScope
        }
        _ => ProbeRefusal::Other,
    }
}
