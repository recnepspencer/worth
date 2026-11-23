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
}

/// One probe's observed outcome: the selected identities, or its refusal.
pub type Observed = Result<Vec<String>, ProbeRefusal>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeRefusal {
    /// The typed managed index refusal.
    Refused(WorthQueryManagedIndexRefusal),
    /// The original request's cancellation, inside the read.
    Cancelled,
    /// Any other refusal.
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
    Ok(())
}

/// The armed probe through the decision's own `index_query`; its outcome is
/// recorded and a refusal stops the decision with its typed cause.
pub fn probe<B>(
    reader: &mut DecisionReader<'_, '_, '_, Catalog, B>,
    own: &str,
) -> Result<(), HandlerExecutionDenial>
where
    B: worth_query_decl::facade::application_operation::ApplicationMutationBinding<Catalog>,
{
    let Some(probe) = PRODUCER_PROBE.get() else {
        return Ok(());
    };
    let directory = match probe {
        ProducerProbe::Own => own.to_owned(),
        ProducerProbe::Peer(peer) => peer.to_owned(),
    };
    let result = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
        directory,
        prefix: String::new(),
    });
    let (observed, result) = match result {
        Ok(rows) => (
            Ok(rows.into_iter().map(|row| row.identity).collect()),
            Ok(()),
        ),
        Err(cause) => match cause.downcast::<WorthQueryManagedIndexDenial>() {
            Ok(denial) => {
                let refusal = match denial.cause() {
                    WorthQueryManagedIndexDenialCause::Refused(refusal) => {
                        ProbeRefusal::Refused(refusal)
                    }
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::Request(
                                worth_execution::WorkCeilingDenial::Stopped(
                                    worth_execution::MapKernelStop::Cancelled,
                                ),
                            ),
                        ),
                    ) => ProbeRefusal::Cancelled,
                    // The producer's coordinated source-query control
                    // observed the original request's cancellation.
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
                    _ => ProbeRefusal::Other,
                };
                (Err(refusal), Err(HandlerExecutionDenial::new(denial)))
            }
            Err(cause) => (Err(ProbeRefusal::Other), Err(cause)),
        },
    };
    OBSERVED.with(|all| all.borrow_mut().push(observed));
    result
}
