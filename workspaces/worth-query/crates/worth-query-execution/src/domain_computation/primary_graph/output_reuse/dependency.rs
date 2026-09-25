use worth_query_declaration::facade::application_query::ApplicationQueryDependencyEquivalence;
use worth_query_installation::facade::WorthQueryInstalledOutputDependencyContract;

use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};

/// An output source must carry an installed meaning and an owner-tracked
/// complete selection. The latter includes negative and absent reads even
/// when the query returned exactly one positive row.
pub(in crate::domain_computation::primary_graph) fn require_installed_output_dependencies<Query>(
    contract: WorthQueryInstalledOutputDependencyContract<'_>,
    source: &WorthQueryObservedSource<Query>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if &source.query_identity != contract.query_identity() {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "output source query meaning differs from the installed dependency contract",
        ));
    }
    if !source.has_complete_output_dependencies() {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
            "output source lacks complete tracked query selection",
        ));
    }
    // The installed graph fixes the bounded projection, predicate, relation,
    // and ordering subfields. Its digest participates in the query identity.
    match contract.equivalence() {
        ApplicationQueryDependencyEquivalence::CompleteNativeSourceRevisions => Ok(()),
    }
}
