use std::{marker::PhantomData, sync::Arc};

use super::{
    binding::WorthQueryCompiledApplicationQuery, WorthQueryApplicationCanonicalArtifact,
    WorthQueryApplicationQueryCanonicalWorkPolicy,
    WorthQueryInstalledApplicationContinuationContract, WorthQueryInstalledApplicationLiveContract,
    WorthQueryInstalledApplicationQueryIdentity, WorthQueryInstalledApplicationReadFamilyBinding,
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledOutputDependencyContract,
};
use crate::{
    application_operation::WorthQueryInstalledAbilityRequirement,
    canonical_work::WorthQueryCanonicalWorkEvidence,
    graph_obligation::{
        WorthQueryInstalledGraphObligationInspection, WorthQueryInstalledGraphObligationSet,
    },
    installed_index::WorthQueryInstalledPackageAuthority,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterDefinition,
    },
    application_schema::ApplicationSchemaBindingIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledApplicationQueryAuthorization {
    Public,
    Ability(WorthQueryInstalledAbilityRequirement),
}

pub struct WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope> {
    compiled: Arc<WorthQueryCompiledApplicationQuery>,
    _marker: PhantomData<fn(Parameters) -> (Schema, Query, QueryResult, Scope)>,
}

/// One retained view of the exact installed Query's immutable obligation set.
/// The already installed compiled Arc owns the rows; sharing it makes no row
/// copies and cannot select obligations from a different Query.
pub struct WorthQueryRetainedApplicationQueryGraphObligations {
    compiled: Arc<WorthQueryCompiledApplicationQuery>,
}

impl WorthQueryRetainedApplicationQueryGraphObligations {
    pub fn inspect(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.compiled.graph_obligations()
    }

    #[doc(hidden)]
    pub fn installed_set(&self) -> &WorthQueryInstalledGraphObligationSet {
        self.compiled.graph_obligation_set()
    }
}

impl std::fmt::Debug for WorthQueryRetainedApplicationQueryGraphObligations {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inspection = self.inspect();
        formatter
            .debug_struct("WorthQueryRetainedApplicationQueryGraphObligations")
            .field("identity", inspection.identity())
            .finish()
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>
{
    pub(crate) fn from_compiled(compiled: Arc<WorthQueryCompiledApplicationQuery>) -> Self {
        Self {
            compiled,
            _marker: PhantomData,
        }
    }
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        self.compiled.binding_identity()
    }
    pub fn identity(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        self.compiled.identity()
    }
    pub fn canonical_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        self.compiled.canonical_basis()
    }
    pub fn canonical_work_policy(&self) -> WorthQueryApplicationQueryCanonicalWorkPolicy {
        self.compiled.canonical_work_policy()
    }
    pub fn installation_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.compiled.installation_canonical_work()
    }
    pub fn authority_identity(&self) -> &str {
        self.compiled.authority_identity()
    }
    pub fn name(&self) -> &str {
        self.compiled.name()
    }
    pub fn scope_entity(&self) -> &str {
        self.compiled.scope_entity()
    }
    pub fn parameter_type(&self) -> &str {
        self.compiled.parameter_type()
    }
    pub fn result_type(&self) -> &str {
        self.compiled.result_type()
    }
    pub fn parameters(&self) -> &[ApplicationQueryParameterDefinition] {
        self.compiled.parameters()
    }
    pub fn read_graph(&self) -> &WorthQueryInstalledGraphReadContract {
        self.compiled.read_graph()
    }

    pub fn output_dependencies(&self) -> WorthQueryInstalledOutputDependencyContract<'_> {
        WorthQueryInstalledOutputDependencyContract::new(
            self.compiled.identity(),
            self.compiled.read_graph(),
            self.compiled.dependency_equivalence(),
        )
    }
    pub fn read_family_binding(&self) -> &WorthQueryInstalledApplicationReadFamilyBinding {
        self.compiled.read_family_binding()
    }
    pub fn continuation(&self) -> Option<&WorthQueryInstalledApplicationContinuationContract> {
        self.compiled.continuation()
    }
    pub fn live(&self) -> Option<&WorthQueryInstalledApplicationLiveContract> {
        self.compiled.live()
    }
    pub fn disclosure(&self) -> &ApplicationQueryDisclosureContract {
        self.compiled.disclosure()
    }
    pub fn authorization(&self) -> &WorthQueryInstalledApplicationQueryAuthorization {
        self.compiled.authorization()
    }
    pub fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.compiled.graph_obligations()
    }
    #[doc(hidden)]
    pub fn retain_graph_obligations_for_admission(&self) -> WorthQueryInstalledGraphObligationSet {
        self.compiled.retain_graph_obligations()
    }

    /// Admit the actual shared-Arc visit before retaining these installed
    /// obligations for a Graph Work plan. No new row backing is allocated.
    pub fn retain_graph_obligations_for_admission_admitted<Stop>(
        &self,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<WorthQueryRetainedApplicationQueryGraphObligations, Stop> {
        admit(1, 0)?;
        Ok(WorthQueryRetainedApplicationQueryGraphObligations {
            compiled: Arc::clone(&self.compiled),
        })
    }
    pub fn basis_support(&self) -> ApplicationQueryBasisSupport {
        self.compiled.basis_support()
    }
    pub fn lanes(&self) -> ApplicationQueryLaneEligibility {
        self.compiled.lanes()
    }
    pub(crate) fn authority_matches(&self, package: &WorthQueryInstalledPackageAuthority) -> bool {
        self.compiled.authority_matches(package)
    }

    /// Before-work bound for the installed identity and seal check performed
    /// by `InstalledApplicationSchema::validate_installed_query`.
    pub fn validation_work_bound(&self) -> Option<u64> {
        super::authority_seal::installed_query_validation_work_bound()
    }
    #[cfg(test)]
    pub(crate) fn shares_compiled_contract_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.compiled, &other.compiled)
    }
}
