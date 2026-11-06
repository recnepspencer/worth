//! An actual declared edge's source read retains its parent and query meaning.
use super::*;
mod verified;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotResult, WorthQueryApplicationOutputDemandSource,
    WorthQueryHistoryOutputReadDenial, WorthQueryOutputDemandSettlement,
};
pub(in crate::domain_computation::primary_graph) use verified::VerifiedHistoryParentCustody;
pub use verified::WorthQueryVerifiedHistoryParent;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;

pub(in crate::domain_computation::primary_graph) struct HistoryDependentSource<
    'authority,
    'phase,
    'scope,
> {
    pub authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
    pub parent: HistoryDependentParent,
    pub connection: std::any::TypeId,
    pub source: &'authority Arc<WorthQueryApplicationReadObservation>,
    pub minimum: &'authority WorthQueryApplicationReadObservation,
    pub output_binding: std::any::TypeId,
    pub discovery_binding: std::any::TypeId,
    pub executed_binding: std::any::TypeId,
    pub query: WorthQueryInstalledApplicationQueryIdentity,
}

/// Only current C admission and the actual installed parent connection can
/// issue this source. It is neither an ordinary child claim nor E qualification.
pub struct WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, Q, R> {
    pub(in crate::domain_computation::primary_graph) source:
        HistoryDependentSource<'authority, 'phase, 'scope>,
    pub(in crate::domain_computation::primary_graph) result:
        WorthQueryApplicationOneShotResult<Q, R>,
}

impl<Q, R> WorthQueryHistoryDependentOutputRead<'_, '_, '_, Q, R> {
    pub fn rows(&self) -> &[R] {
        self.result.rows()
    }

    /// Discovery has only readonly disclosure and dependency meaning. Output
    /// source reads must instead be consumed by their exact child admission.
    pub fn into_discovery(
        self,
    ) -> Result<WorthQueryHistoryOutputDiscovery<Q, R>, WorthQueryHistoryOutputReadDenial> {
        if self.source.executed_binding != self.source.discovery_binding {
            return Err(WorthQueryHistoryOutputReadDenial::Demand(
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "this history read is not the declared discovery binding",
                ),
            ));
        }
        Ok(WorthQueryHistoryOutputDiscovery {
            source: self
                .result
                .into_admitted_disclosed()
                .into_output_demand_source(),
            profile: self.source.authority.profile().clone(),
            original: Arc::clone(self.source.authority.original()),
            parent: self.source.parent,
            connection: self.source.connection,
            binding: self.source.discovery_binding,
            query: self.source.query,
            basis: Arc::clone(self.source.source),
        })
    }
}

/// Readonly discovery data and complete original dependencies. There is no
/// conversion to a raw output claim, an ordinary demand or a qualified endpoint.
///
/// ```compile_fail,E0308
/// use worth_query_execution::{
///     facade::primary_graph::WorthQueryApplicationOutputDemandSource,
///     publication_boundary::document_history::WorthQueryHistoryOutputDiscovery,
/// };
/// fn bypass<Q, R>(discovery: WorthQueryHistoryOutputDiscovery<Q, R>)
///     -> WorthQueryApplicationOutputDemandSource<Q, R> {
///     discovery
/// }
/// ```
///
/// Read the disclosed rows while preserving their discovery-only custody.
/// ```
/// use worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputDiscovery;
/// fn rows<Q, R>(discovery: &WorthQueryHistoryOutputDiscovery<Q, R>) -> &[R] {
///     discovery.rows()
/// }
/// ```
pub struct WorthQueryHistoryOutputDiscovery<Q, R> {
    pub(in crate::domain_computation::primary_graph) source:
        WorthQueryApplicationOutputDemandSource<Q, R>,
    pub(in crate::domain_computation::primary_graph) profile: WorthQueryHistoryOutputProfile,
    pub(in crate::domain_computation::primary_graph) original:
        Arc<WorthQueryApplicationReadObservation>,
    pub(in crate::domain_computation::primary_graph) parent: HistoryDependentParent,
    pub(in crate::domain_computation::primary_graph) connection: std::any::TypeId,
    pub(in crate::domain_computation::primary_graph) binding: std::any::TypeId,
    pub(in crate::domain_computation::primary_graph) query:
        WorthQueryInstalledApplicationQueryIdentity,
    pub(in crate::domain_computation::primary_graph) basis:
        Arc<WorthQueryApplicationReadObservation>,
}

impl<Q, R> WorthQueryHistoryOutputDiscovery<Q, R> {
    pub fn rows(&self) -> &[R] {
        self.source.rows()
    }
}

/// Exact parent identity carried by a read; neither posture converts to the other.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum HistoryDependentParent {
    Settled(Arc<WorthQueryOutputDemandSettlement>),
    Verified(Arc<VerifiedHistoryParentCustody>),
}
impl HistoryDependentParent {
    pub(in crate::domain_computation::primary_graph) fn same_custody(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Settled(a), Self::Settled(b)) => Arc::ptr_eq(a, b),
            (Self::Verified(a), Self::Verified(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
