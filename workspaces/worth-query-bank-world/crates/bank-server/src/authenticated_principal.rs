use bank_domain::model::BankPrincipalId;
use bank_domain::schema::{BankSchema, Principal};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryPrincipalAttribute;
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::primary_graph::WorthQueryAuthenticatedPrincipal;

/// Bank-owned authenticated actor carrying both Query identity authority and
/// the typed bank principal identity resolved from the same installed mapping.
pub struct BankAuthenticatedPrincipal {
    principal_id: BankPrincipalId,
    external: WorthQueryAuthenticatedExternalPrincipal<BankSchema>,
    query: WorthQueryAuthenticatedPrincipal<BankSchema, Principal, BankPrincipalId>,
}

impl std::fmt::Debug for BankAuthenticatedPrincipal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankAuthenticatedPrincipal")
            .finish_non_exhaustive()
    }
}

impl BankAuthenticatedPrincipal {
    pub(crate) const fn new(
        principal_id: BankPrincipalId,
        external: WorthQueryAuthenticatedExternalPrincipal<BankSchema>,
        query: WorthQueryAuthenticatedPrincipal<BankSchema, Principal, BankPrincipalId>,
    ) -> Self {
        Self {
            principal_id,
            external,
            query,
        }
    }

    pub const fn principal_id(&self) -> BankPrincipalId {
        self.principal_id
    }

    pub fn external_identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        self.external.identity()
    }

    pub fn attributes(&self) -> &[WorthQueryPrincipalAttribute] {
        self.external.attributes()
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.query.examined_candidate_count()
    }

    /// Monotonic deadline after which this authentication cannot authorize
    /// new or retained work in the current process.
    pub fn authentication_valid_until(&self) -> std::time::Instant {
        self.external.valid_until()
    }

    pub(crate) const fn external(&self) -> &WorthQueryAuthenticatedExternalPrincipal<BankSchema> {
        &self.external
    }

    pub(crate) const fn query(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<BankSchema, Principal, BankPrincipalId> {
        &self.query
    }
}
