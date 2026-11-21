use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use super::super::super::InstalledProducerEdition;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryRequestAffinity;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use worth_runtime_world::facade::ProductBranchObservation;

// Construction stays in disclosure validation. In particular, no arbitrary
// value paired with a descriptive observation can mint either predecessor.
pub struct AdmittedDisclosure<
    Query,
    Value,
> {
    pub value: Value,
    pub source: WorthQueryObservedSource<Query>,
    pub affinity: WorthQueryApplicationQueryRequestAffinity,
    // Provenance of source admission; never the final executor seal.
    pub admission_edition: InstalledProducerEdition,
}

pub struct FreshOutputDisclosure<
    Query,
    Value,
>(pub AdmittedDisclosure<Query, Value>);

pub struct RetainedProgramOutputDisclosure<
    Query,
    Value,
>(pub AdmittedDisclosure<Query, Value>);

pub enum ValidatedOutputDisclosure<
    Query,
    Value,
> {
    Fresh(FreshOutputDisclosure<Query, Value>),
    RetainedProgram(RetainedProgramOutputDisclosure<Query, Value>),
}

impl<Query, Value> ValidatedOutputDisclosure<Query, Value> {
    fn admitted(&self) -> &AdmittedDisclosure<Query, Value> {
        match self {
            Self::Fresh(proof) => &proof.0,
            Self::RetainedProgram(proof) => &proof.0,
        }
    }

    pub fn value(
        &self,
    ) -> &Value {
        &self.admitted().value
    }

    pub fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.admitted().source
    }
}

impl<Query, Value> FreshOutputDisclosure<Query, Value> {
    /// Borrow the same owner-issued disclosure while its observed source is
    /// retained for required successor admission. The proof itself remains
    /// intact for the later Scheduleâ†’Execute transition.
    pub fn value(
        &self,
    ) -> &Value {
        &self.0.value
    }

    pub fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.0.source
    }

    pub fn into_parts(
        self,
    ) -> (Value, WorthQueryObservedSource<Query>) {
        (self.0.value, self.0.source)
    }

    pub fn admits<
        Schema,
    >(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        selected: &ProductBranchObservation,
    ) -> bool {
        self.0.affinity.admits(principal, request)
            && matches!(&self.0.source.selection,
                crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(disclosed) if disclosed.matches_observation(selected))
    }
}

impl<Query, Value> RetainedProgramOutputDisclosure<Query, Value> {
    pub fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.0.source
    }

    pub fn admits<
        Schema,
    >(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> bool {
        self.0.affinity.admits(principal, request)
    }
}
