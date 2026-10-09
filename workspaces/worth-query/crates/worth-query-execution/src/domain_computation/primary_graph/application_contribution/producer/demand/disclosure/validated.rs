use std::any::Any;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

use super::super::super::InstalledProducerEdition;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryRequestAffinity;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchObservation;

use crate::domain_computation::primary_graph::InvalidationEditAdmission;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum FreshDisclosureAdmissionStop
{
    Admission(CompanionPreflightStop),
    AccountingOverflow,
}

// Construction stays in disclosure validation. In particular, no arbitrary
// value paired with a descriptive observation can mint either predecessor.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct AdmittedDisclosure<
    Query,
    Value,
> {
    pub(super) value: Value,
    pub(super) source: WorthQueryObservedSource<Query>,
    pub(super) affinity: WorthQueryApplicationQueryRequestAffinity,
    pub(super) edition: InstalledProducerEdition,
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct FreshOutputDisclosure<
    Query,
    Value,
>(pub(super) AdmittedDisclosure<Query, Value>);

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct RetainedProgramOutputDisclosure<
    Query,
    Value,
>(pub(super) AdmittedDisclosure<Query, Value>);

pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum ValidatedOutputDisclosure<
    Query,
    Value,
> {
    Fresh(FreshOutputDisclosure<Query, Value>),
    RetainedProgram(RetainedProgramOutputDisclosure<Query, Value>),
}

/// Borrowed transport of the entire owner-issued proof. Its edition guard is
/// inspected before runtime erasure is opened by an installed executor.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ValidatedProducerInput<
    'a,
> {
    edition: InstalledProducerEdition,
    proof: &'a mut dyn Any,
}

impl<Query, Value> ValidatedOutputDisclosure<Query, Value> {
    fn admitted(&self) -> &AdmittedDisclosure<Query, Value> {
        match self {
            Self::Fresh(proof) => &proof.0,
            Self::RetainedProgram(proof) => &proof.0,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn value(
        &self,
    ) -> &Value {
        &self.admitted().value
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.admitted().source
    }
}

impl<Query: 'static, Value: 'static> ValidatedOutputDisclosure<Query, Value> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn with_erased<
        Result,
    >(
        self,
        execute: impl FnOnce(ValidatedProducerInput<'_>) -> Result,
    ) -> Result {
        let edition = self.admitted().edition;
        let mut proof = Some(self);
        execute(ValidatedProducerInput {
            edition,
            proof: &mut proof,
        })
    }
}

impl ValidatedProducerInput<'_> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn take<
        Query: 'static,
        Value: 'static,
    >(
        self,
        installed: InstalledProducerEdition,
    ) -> Option<ValidatedOutputDisclosure<Query, Value>> {
        if self.edition != installed {
            return None;
        }
        self.proof
            .downcast_mut::<Option<ValidatedOutputDisclosure<Query, Value>>>()?
            .take()
    }
}
impl<Query, Value> FreshOutputDisclosure<Query, Value> {
    /// Recheck the actual selected execution operands under its carried meter.
    /// The original disclosure acceptance remains the sole semantic check.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn admits_selected_admitted<
        Schema,
    >(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        selected: &ProductBranchObservation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, FreshDisclosureAdmissionStop> {
        use FreshDisclosureAdmissionStop as Stop;
        admission.charge_external_work(6).map_err(Stop::Admission)?;
        let metadata = self
            .0
            .affinity
            .comparison_metadata_visits(principal)
            .ok_or(Stop::AccountingOverflow)?;
        admission
            .charge_external_work(metadata)
            .map_err(Stop::Admission)?;
        let affinity = self
            .0
            .affinity
            .comparison_work(principal)
            .ok_or(Stop::AccountingOverflow)?;
        // `matches_observation` compares both branch-name operands and the
        // fixed read-identity axes. World current-head pricing only covers one
        // name, so admit this selected disclosure comparison independently.
        let product = match &self.0.source.selection {
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Relational => 1,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(disclosed) => {
                let names = disclosed.branch_identity().name().as_str().len()
                    .checked_add(selected.branch_identity().name().as_str().len())
                    .ok_or(Stop::AccountingOverflow)?;
                let fixed = std::mem::size_of::<crate::basis::WorthQueryProductBranchReadIdentity>()
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(8))
                    .ok_or(Stop::AccountingOverflow)?;
                u64::try_from(names.checked_add(fixed).ok_or(Stop::AccountingOverflow)?)
                    .map_err(|_| Stop::AccountingOverflow)?
            }
        };
        let total = affinity
            .checked_add(product)
            .and_then(|work| work.checked_add(4))
            .ok_or(Stop::AccountingOverflow)?;
        admission
            .charge_external_work(total)
            .map_err(Stop::Admission)?;
        Ok(self.admits(principal, request, selected))
    }

    /// Borrow the same owner-issued disclosure while its observed source is
    /// retained for required successor admission. The proof itself remains
    /// intact for the later Schedule→Execute transition.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn value(
        &self,
    ) -> &Value {
        &self.0.value
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.0.source
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn into_parts(
        self,
    ) -> (Value, WorthQueryObservedSource<Query>) {
        (self.0.value, self.0.source)
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn admits<
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
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.0.source
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn admits<
        Schema,
    >(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> bool {
        self.0.affinity.admits(principal, request)
    }
}
