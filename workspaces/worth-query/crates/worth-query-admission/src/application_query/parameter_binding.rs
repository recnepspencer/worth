use worth_foundational::facade::{AspectValue, CanonicalDigestDerivationDenial, CanonicalDigestId};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::{
    parameter_canonical_basis::prepare_parameter_basis,
    WorthQueryApplicationParameterCanonicalArtifact,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryParameterDenialKind {
    ParameterSetMismatch,
    ParameterTypeMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryParameterDenial {
    kind: WorthQueryApplicationQueryParameterDenialKind,
    parameter: String,
}

impl WorthQueryApplicationQueryParameterDenial {
    fn new(
        kind: WorthQueryApplicationQueryParameterDenialKind,
        parameter: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            parameter: parameter.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryParameterDenialKind {
        self.kind
    }

    pub fn parameter(&self) -> &str {
        &self.parameter
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAdmittedApplicationQueryParameters {
    canonical: WorthQueryApplicationParameterCanonicalArtifact,
    bindings: Vec<(&'static str, AspectValue)>,
    budget: worth_foundational::facade::CanonicalDigestWorkBudget,
}

impl WorthQueryAdmittedApplicationQueryParameters {
    pub const fn identity(&self) -> &CanonicalDigestId {
        self.canonical.identity()
    }

    pub fn canonical_basis(&self) -> &WorthQueryApplicationParameterCanonicalArtifact {
        &self.canonical
    }

    pub fn bindings(&self) -> &[(&'static str, AspectValue)] {
        &self.bindings
    }

    /// Compares a mutation's declared selectors with this admitted query's exact
    /// canonical basis, under the same installed canonicalization budget.
    pub fn matches_expected<Query>(
        &self,
        expected: ApplicationQueryParameterSet<Query>,
    ) -> Result<bool, WorthQueryApplicationQueryParameterDenial> {
        let mut bindings = expected.bindings().to_vec();
        bindings.sort_by_key(|(name, _)| *name);
        let canonical = prepare_parameter_basis(&bindings, self.budget)
            .map_err(|denial| canonical_work_denial("mutation-source", denial))?;
        Ok(self.canonical.is_equivalent_to(&canonical))
    }
}

pub fn admit_application_query_parameters<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    if parameters.bindings().len() != query.parameters().len() {
        return Err(WorthQueryApplicationQueryParameterDenial::new(
            WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
            query.name(),
        ));
    }
    let mut bindings = parameters.bindings().to_vec();
    bindings.sort_by_key(|(name, _)| *name);
    for declared in query.parameters() {
        let Some((_, value)) = bindings.iter().find(|(name, _)| *name == declared.name()) else {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
                declared.name(),
            ));
        };
        if value.value_family() != declared.scalar_family() {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterTypeMismatch,
                declared.name(),
            ));
        }
    }
    let canonical = prepare_parameter_basis(&bindings, query.canonical_work_policy().parameters())
        .map_err(|denial| canonical_work_denial(query.name(), denial))?;
    Ok(WorthQueryAdmittedApplicationQueryParameters {
        canonical,
        bindings,
        budget: query.canonical_work_policy().parameters(),
    })
}

fn canonical_work_denial(
    parameter: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryApplicationQueryParameterDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryApplicationQueryParameterDenialKind::CanonicalEntryBudgetExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryApplicationQueryParameterDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        _ => WorthQueryApplicationQueryParameterDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryApplicationQueryParameterDenial::new(kind, parameter)
}
