#[cfg(any(test, feature = "certification-query-lookup"))]
use worth_query_declaration::facade::application_query::ApplicationQueryReference;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterDefinition,
        ErasedApplicationQueryDefinition,
    },
    application_schema::ApplicationSchemaBindingIdentity,
    portable_identity::WorthQueryPortableTypeIdentity,
};

use crate::{
    application_query::{
        WorthQueryApplicationCanonicalArtifact, WorthQueryApplicationQueryCanonicalWorkPolicy,
        WorthQueryInstalledApplicationContinuationContract,
        WorthQueryInstalledApplicationLiveContract,
        WorthQueryInstalledApplicationQueryAuthorization,
        WorthQueryInstalledApplicationQueryIdentity,
        WorthQueryInstalledApplicationReadFamilyBinding, WorthQueryInstalledGraphReadContract,
    },
    authority_cryptography::AuthoritySeal,
    canonical_work::WorthQueryCanonicalWorkEvidence,
    graph_obligation::{
        WorthQueryInstalledGraphObligationInspection, WorthQueryInstalledGraphObligationSet,
    },
    installed_index::WorthQueryInstalledPackageAuthority,
};

pub(crate) struct WorthQueryCompiledApplicationQuery {
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) canonical: WorthQueryApplicationCanonicalArtifact,
    pub(super) canonical_work_policy: WorthQueryApplicationQueryCanonicalWorkPolicy,
    pub(super) installation_canonical_work: WorthQueryCanonicalWorkEvidence,
    pub(super) identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(super) authority_identity: AuthoritySeal,
    pub(super) definition: ErasedApplicationQueryDefinition,
    pub(super) parameter_type: WorthQueryPortableTypeIdentity,
    pub(super) result_type: WorthQueryPortableTypeIdentity,
    pub(super) read_family: WorthQueryInstalledApplicationReadFamilyBinding,
    pub(super) continuation: Option<WorthQueryInstalledApplicationContinuationContract>,
    pub(super) live: Option<WorthQueryInstalledApplicationLiveContract>,
    pub(super) authorization: WorthQueryInstalledApplicationQueryAuthorization,
    pub(super) obligations: WorthQueryInstalledGraphObligationSet,
}

impl WorthQueryCompiledApplicationQuery {
    pub(crate) fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub(crate) fn identity(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.identity
    }

    pub(crate) fn canonical_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        &self.canonical
    }

    pub(crate) const fn canonical_work_policy(
        &self,
    ) -> WorthQueryApplicationQueryCanonicalWorkPolicy {
        self.canonical_work_policy
    }

    pub(crate) const fn installation_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.installation_canonical_work
    }

    pub(crate) fn authority_identity(&self) -> &str {
        self.authority_identity.as_str()
    }

    pub(crate) fn name(&self) -> &str {
        self.definition.name()
    }

    pub(crate) fn scope_entity(&self) -> &str {
        self.definition.scope_entity()
    }

    pub(crate) fn parameter_type(&self) -> &str {
        self.parameter_type.as_str()
    }

    pub(crate) fn result_type(&self) -> &str {
        self.result_type.as_str()
    }

    pub(crate) fn parameters(&self) -> &[ApplicationQueryParameterDefinition] {
        self.definition.parameters()
    }

    pub(crate) fn read_graph(&self) -> &WorthQueryInstalledGraphReadContract {
        self.read_family.planning_contract()
    }

    pub(crate) const fn read_family_binding(
        &self,
    ) -> &WorthQueryInstalledApplicationReadFamilyBinding {
        &self.read_family
    }

    pub(crate) const fn continuation(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationContinuationContract> {
        self.continuation.as_ref()
    }

    pub(crate) const fn live(&self) -> Option<&WorthQueryInstalledApplicationLiveContract> {
        self.live.as_ref()
    }

    pub(crate) fn disclosure(&self) -> &ApplicationQueryDisclosureContract {
        self.definition.disclosure()
    }

    pub(crate) const fn authorization(&self) -> &WorthQueryInstalledApplicationQueryAuthorization {
        &self.authorization
    }

    pub(crate) const fn graph_obligations(
        &self,
    ) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.obligations.inspect()
    }

    pub(crate) fn retain_graph_obligations(&self) -> WorthQueryInstalledGraphObligationSet {
        self.obligations.clone()
    }

    pub(crate) fn basis_support(&self) -> ApplicationQueryBasisSupport {
        self.definition.basis_support()
    }

    pub(crate) fn lanes(&self) -> ApplicationQueryLaneEligibility {
        self.definition.lanes()
    }

    pub(crate) fn authority_matches(&self, package: &WorthQueryInstalledPackageAuthority) -> bool {
        super::super::authority_seal::verify_installed_query_authority_seal(
            &self.authority_identity,
            &package.authority_key,
            &self.binding_identity,
            &self.identity,
            self.obligations.identity(),
        )
    }

    #[cfg(any(test, feature = "certification-query-lookup"))]
    pub(crate) fn matches_reference<Schema, Query, Parameters, QueryResult, Scope>(
        &self,
        reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> bool {
        self.definition.matches_reference(reference)
    }
}
