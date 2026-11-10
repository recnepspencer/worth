//! Derivation material belongs to the original occurrence's installed mechanism.
pub(super) mod recorded_input;
use super::{
    disclosure::WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource,
};

use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionPreImage,
        ApplicationCorrectionSourceMaterial, DeclaredPreImageLocus,
    },
    application_operation::ApplicationMutationBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    InstalledCorrectionAuthority, InstalledCorrectionMechanism,
};
pub(crate) mod source_outputs;
pub use source_outputs::WorthQueryCorrectionOutputProjectionDenial;

pub(super) fn mechanism<'a, S: ApplicationSchema>(
    source: &'a WorthQuerySelectedCorrectionSource<'_, S>,
) -> Result<&'a InstalledCorrectionMechanism, Denial> {
    let contract = source
        .occurrence()
        .aftermath()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    match contract.authority() {
        InstalledCorrectionAuthority::RuntimeAlone => {}
        InstalledCorrectionAuthority::RuntimeWithExternalOwner
            if contract
                .completed_external_reconciliation()
                .is_some_and(|declared| {
                    contract.reconciliation().is_some_and(|installed| {
                        declared.procedure_slot() == installed.procedure_slot()
                    })
                })
                && contract.correction_disclosure().is_some()
                && matches!(
                    contract.mechanism(),
                    Some(InstalledCorrectionMechanism::Compensation(_))
                )
                && source
                    .occurrence()
                    .memory()
                    .and_then(|value| value.commit_evidence().workflow_origin())
                    .is_some() =>
        {
            match source.application.resolve_correction_external_settlement(source.occurrence().memory().expect("workflow origin belongs to real Memory evidence")) {
                Ok(true) => {},
                Ok(false) => return Err(Denial::Workflow(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
                        "original external workflow operation has no authenticated completion",
                    ),
                )),
                Err(_) => return Err(Denial::UnsupportedSourceMeaning),
            }
        }
        InstalledCorrectionAuthority::RuntimeWithExternalOwner
        | InstalledCorrectionAuthority::NotCorrectable => {
            return Err(Denial::UnsupportedSourceMeaning)
        }
    }
    contract.mechanism().ok_or(Denial::UnsupportedSourceMeaning)
}

pub(super) fn derive<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
    source: &WorthQuerySelectedCorrectionSource<'_, S>,
    request: worth_execution::ExecutionRequest<'_, '_>,
    input: &<C::Source as ApplicationMutationBinding<S>>::Input,
) -> Result<C::InverseIntent, Denial> {
    match mechanism(source)? {
        InstalledCorrectionMechanism::RecordedInverse(inverse)
            if inverse.preimage_demand().is_empty() =>
        {
            let outputs = source_outputs::SourceOutputs::<S, C>::new(source, request);
            let result = C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::CreatedEntity(&outputs),
            );
            finish_derivation(result, outputs.take_failure())
        }
        InstalledCorrectionMechanism::RecordedInverse(_) => {
            if !source.occurrence().has_preimage() {
                return Err(Denial::MissingPreImage);
            }
            C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::RecordedInverse(&DisclosedPreImage::Retained(
                    source.occurrence(),
                )),
            )
            .map_err(Denial::Domain)
        }
        InstalledCorrectionMechanism::Compensation(_) => {
            let outputs = source_outputs::SourceOutputs::<S, C>::new(source, request);
            let result = C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::Compensation(&outputs),
            );
            finish_derivation(result, outputs.take_failure())
        }
    }
}

pub fn finish_derivation<T>(
    result: Result<T, worth_query_declaration::facade::application_aftermath::ApplicationCorrectionDerivationDenial>,
    failure: Option<WorthQueryCorrectionOutputProjectionDenial>,
) -> Result<T, Denial> {
    if let Some(cause) = failure {
        return Err(Denial::SourceOutput(cause));
    }
    result.map_err(Denial::Domain)
}

enum DisclosedPreImage<'a> {
    Retained(&'a super::occurrence::CorrectionOccurrence),
    Captured(&'a super::WorthQueryPrimaryGraphCommittedApplication),
}
impl DisclosedPreImage<'_> {
    fn retained(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        match self {
            Self::Retained(value) => value.memory_preimage(),
            Self::Captured(value) => value.commit_evidence().retained_preimage(),
        }
    }
    fn outputs(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        match self {
            Self::Retained(value) => value.correspondence(),
            Self::Captured(value) => value.commit_evidence().output_correspondence(),
        }
    }
}
impl<S> ApplicationCorrectionPreImage<S> for DisclosedPreImage<'_> {
    fn output_entity(
        &self,
        role: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor,
    ) -> Option<worth_foundational::facade::EntityId> {
        let mut roles =
            self.outputs()
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let (_, _, _, id) = roles.next()?;
        if roles.next().is_some() {
            return None;
        }
        Some(historical_entity(id))
    }

    fn relation_count(
        &self,
        locus: &worth_query_declaration::facade::application_aftermath::DeclaredPreImageRelationLocus<S>,
    ) -> Option<usize> {
        let retained = self.retained()?;
        if !retained.relation_loci().iter().any(|value| {
            value.relation() == locus.relation()
                && value.from() == locus.from()
                && value.to() == locus.to()
        }) {
            return None;
        }
        Some(
            retained
                .relations()
                .iter()
                .filter(|row| {
                    row.locus().relation() == locus.relation()
                        && row.locus().from() == locus.from()
                        && row.locus().to() == locus.to()
                })
                .count(),
        )
    }
    fn relation_at(&self, locus: &worth_query_declaration::facade::application_aftermath::DeclaredPreImageRelationLocus<S>, index: usize) -> Option<worth_query_declaration::facade::application_aftermath::ApplicationCorrectionRelationMaterial>{
        let row = self
            .retained()?
            .relations()
            .iter()
            .filter(|row| {
                row.locus().relation() == locus.relation()
                    && row.locus().from() == locus.from()
                    && row.locus().to() == locus.to()
            })
            .nth(index)?;
        let describe = |(from, to)| (historical_entity(from), historical_entity(to));
        Some(worth_query_declaration::facade::application_aftermath::ApplicationCorrectionRelationMaterial::from_historical_endpoints(row.mutation().before().map(describe), row.mutation().after().map(describe)))
    }

    fn field(
        &self,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        match self {
            Self::Retained(value) => {
                value.preimage_value(locus.entity(), locus.aspect(), locus.field())
            }
            Self::Captured(_) => self
                .retained()?
                .fields()
                .iter()
                .find(|field| {
                    field.locus().entity() == locus.entity()
                        && field.locus().aspect() == locus.aspect()
                        && field.locus().field() == locus.field()
                })
                .map(|field| field.value()),
        }
    }
    fn field_for_output(
        &self,
        role: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        if role.entity() != locus.entity() {
            return None;
        }
        let mut roles =
            self.outputs()
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let (_, _, _, original) = roles.next()?;
        if roles.next().is_some() {
            return None;
        }
        if let Self::Retained(value) = self {
            return value.preimage_key(
                original,
                locus.entity(),
                &worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Planned,
                    worth_foundational::facade::AspectKey::new(locus.aspect())?,
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new(locus.field())?,
                    ),
                ),
            );
        }
        self.retained()?
            .fields()
            .iter()
            .find(|field| {
                field.target_record()
                    == &worth_relational::facade::transactions::RecordRef::Entity(original)
                    && field.locus().entity() == locus.entity()
                    && field.locus().aspect() == locus.aspect()
                    && field.locus().field() == locus.field()
            })
            .map(|field| field.value())
    }
}

/// The installed correction derives finite demands from its original input and pre-image.
pub(super) fn capture_coupled<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
    material: &super::occurrence::CorrectionOccurrence,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands{
    use crate::domain_computation::primary_graph::{
        application_attempt::coupled_outputs, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind,
    };
    if material.memory().is_none() {
        let Some((bytes, identity)) = material.input() else {
            return coupled_outputs::DeclaredCoupledOutputDemands::failed(Attempt::new(
                Kind::CorrectionCaptureDenied,
                C::IDENTITY,
            ));
        };
        let Ok(input) = recorded_input::recover_bytes(bytes, identity, C::recover_source_input)
        else {
            return coupled_outputs::DeclaredCoupledOutputDemands::failed(Attempt::new(
                Kind::CorrectionCaptureDenied,
                C::IDENTITY,
            ));
        };
        return coupled_outputs::capture_correction::<S, C>(
            execution,
            input.input(),
            &DisclosedPreImage::Retained(material),
        );
    }
    let material = material.memory().expect("actual Memory occurrence");
    let Some(input) = material
        .commit_evidence()
        .original_authority()
        .retained_governed_input_carrier()
        .and_then(|carrier| {
            carrier.retained_input::<<C::Source as ApplicationMutationBinding<S>>::InputBinding>()
        })
    else {
        return coupled_outputs::DeclaredCoupledOutputDemands::failed(Attempt::new(
            Kind::CorrectionCaptureDenied,
            C::IDENTITY,
        ));
    };
    let Ok(input) = recorded_input::recover(input, C::recover_source_input) else {
        return coupled_outputs::DeclaredCoupledOutputDemands::failed(Attempt::new(
            Kind::CorrectionCaptureDenied,
            C::IDENTITY,
        ));
    };
    coupled_outputs::capture_correction::<S, C>(
        execution,
        input.input(),
        &DisclosedPreImage::Captured(material),
    )
}

fn historical_entity(
    id: worth_relational::facade::identity::EntityId,
) -> worth_foundational::facade::EntityId {
    worth_foundational::facade::EntityId::new(
        worth_foundational::facade::PartitionId(id.partition_value()),
        id.local_slot_value(),
        id.generation_value(),
    )
}
