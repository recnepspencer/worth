//! Retained and external emissions admitted into one effect program.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationStructuredValueBinding,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone)]
pub struct WorthQueryApplicationEmission {
    effect: &'static str,
    payload_type: WorthQueryPortableTypeIdentity,
    payload_type_id: TypeId,
    payload: Arc<dyn Any + Send + Sync>,
    external_payload: Option<WorthQueryExternalPayloadProjection>,
    _native_query_backing: Option<
        crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext,
    >,
    _native_values_backing:
        Option<crate::domain_computation::primary_graph::operation_body::value::PreparedBodyValues>,
}

#[derive(Clone)]
struct WorthQueryExternalPayloadProjection {
    bytes: Arc<[u8]>,
    maximum_bytes: u64,
}

impl WorthQueryApplicationEmission {
    pub fn new<Binding>(effect: &'static str, payload: Binding::Value) -> Self
    where
        Binding: ApplicationStructuredValueBinding,
        Binding::Value: Send + Sync,
    {
        Self {
            effect,
            payload_type: Binding::IDENTITY,
            payload_type_id: TypeId::of::<Binding::Value>(),
            payload: Arc::new(payload),
            external_payload: None,
            _native_query_backing: None,
            _native_values_backing: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn new_external<Binding>(
        effect: &'static str,
        payload: Binding::Value,
    ) -> Result<Self, ()>
    where
        Binding: ApplicationExternalEffectBinding,
        Binding::Value: Send + Sync,
    {
        let projection = worth_query_declaration::facade::application_schema::ApplicationExternalEffectPayloadProjection::encode::<Binding>(effect, payload).map_err(|_| ())?;
        Ok(Self::from_external_projection(projection, None, None))
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_lifecycle(
        derived: &worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect,
    ) -> Self {
        Self {
            effect: derived.effect(),
            payload_type: derived.payload_identity(),
            payload_type_id: TypeId::from(derived.payload_type_id()),
            payload: derived.payload(),
            external_payload: None,
            _native_query_backing: None,
            _native_values_backing: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn is_exact_lifecycle(
        &self,
        derived: &worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect,
    ) -> bool {
        self.effect == derived.effect()
            && self.payload_type == derived.payload_identity()
            && self.payload_type_id == TypeId::from(derived.payload_type_id())
            && derived.payload_is(&self.payload)
            && self.is_well_formed()
    }

    pub(in crate::domain_computation::primary_graph) fn is_well_formed(&self) -> bool {
        !self.effect.is_empty()
            && self.payload_type.is_valid()
            && self.payload_type_id == TypeId::from(self.payload.as_ref().type_id())
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn effect_name(
        &self,
    ) -> &str {
        self.effect
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_external_projection(
        projection: worth_query_declaration::facade::application_schema::ApplicationExternalEffectPayloadProjection,
        backing: Option<crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>,
        values: Option<
            crate::domain_computation::primary_graph::operation_body::value::PreparedBodyValues,
        >,
    ) -> Self {
        Self {
            effect: projection.effect(),
            payload_type: projection.identity().clone(),
            payload_type_id: TypeId::from(projection.value_type()),
            payload: projection.value(),
            external_payload: Some(WorthQueryExternalPayloadProjection {
                bytes: projection.bytes(),
                maximum_bytes: projection.maximum_bytes(),
            }),
            _native_query_backing: backing,
            _native_values_backing: values,
        }
    }

    fn external_payload(&self) -> Option<&WorthQueryExternalPayloadProjection> {
        self.external_payload.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn payload_ref<Schema, Effect, Payload>(
        &self,
        effect: &worth_query_declaration::facade::application_schema::ApplicationEffectRef<
            Schema,
            Effect,
            Payload,
        >,
    ) -> Option<&Payload>
    where
        Payload: 'static,
    {
        (self.effect == effect.name())
            .then(|| self.payload.downcast_ref::<Payload>())
            .flatten()
    }

    pub(crate) const fn effect(&self) -> &'static str {
        self.effect
    }

    pub(crate) fn payload<Payload: 'static>(&self) -> Option<&Payload> {
        self.payload.downcast_ref()
    }
}

pub struct WorthQueryAdmittedApplicationEmissionBatch {
    emissions: Vec<WorthQueryApplicationEmission>,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryAdmittedApplicationEmissionBatch {
    pub(in crate::domain_computation::primary_graph) fn admit(
        emissions: Vec<WorthQueryApplicationEmission>,
    ) -> Result<Self, &'static str> {
        if emissions.iter().any(|emission| !emission.is_well_formed()) {
            return Err("application commit carried an invalid typed emission");
        }
        Ok(Self {
            emissions,
            memory: None,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn into_live_emissions(
        self,
    ) -> (
        Vec<WorthQueryApplicationEmission>,
        Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        (self.emissions, self.memory)
    }
    pub(in crate::domain_computation::primary_graph) fn combine<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        batches: impl Iterator<Item = &'a Self> + Clone,
    ) -> Result<Self, crate::domain_computation::WorthQueryProviderSessionCommitStop> {
        use crate::domain_computation::{
            WorthQueryProviderSessionCommitStop as Stop,
            WorthQueryProviderSessionDenialKind as Kind,
            WorthQueryProviderSessionFailure as Failure,
            WorthQueryProviderSessionProtocolStage as Stage,
        };
        let denied = || {
            Stop::PreEffectDenied(Failure::new(
                Kind::AllocationDenied,
                Stage::Commit,
                "coupled emission backing could not allocate",
                Default::default(),
            ))
        };
        let count = batches
            .clone()
            .try_fold(0usize, |count, batch| count.checked_add(batch.len()))
            .ok_or_else(denied)?;
        let bytes = count
            .checked_mul(std::mem::size_of::<WorthQueryApplicationEmission>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(denied)?;
        let mut memory = execution.reserve_retained_memory(bytes).map_err(crate::domain_computation::primary_graph::provider::relational_execution_denial::request_commit_stop)?;
        let mut emissions = Vec::new();
        emissions.try_reserve_exact(count).map_err(|_| denied())?;
        if let Some(memory) = memory.as_mut() {
            memory.resize((emissions.capacity() * std::mem::size_of::<WorthQueryApplicationEmission>()) as u64).map_err(|cause| crate::domain_computation::primary_graph::provider::relational_execution_denial::request_commit_stop(worth_execution::WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(cause))))?;
        }
        execution.run_native_with_checkpoint(|context| {
            for batch in batches { for emission in &batch.emissions { context.checkpoint(1)?; emissions.push(emission.clone()); } }
            Ok::<_, worth_execution::MapKernelStop>(())
        }).map_err(crate::domain_computation::primary_graph::provider::relational_execution_denial::request_commit_stop)?.0.map_err(|cause| crate::domain_computation::primary_graph::provider::relational_execution_denial::request_commit_stop(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        Ok(Self { emissions, memory })
    }

    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.emissions.len()
    }

    pub(in crate::domain_computation::primary_graph) fn external_payload(
        &self,
        contract: &worth_query_installation::facade::InstalledExternalEffectContract,
    ) -> Result<Option<Vec<u8>>, &'static str> {
        let worth_query_installation::facade::InstalledExternalEffectContract::Declared {
            effect,
            rust_payload_type,
            maximum_payload_bytes,
            ..
        } = contract
        else {
            return Ok(None);
        };
        let mut matching = self.emissions.iter().filter(|emission| {
            emission.effect == effect && emission.payload_type == *rust_payload_type
        });
        let emission = matching
            .next()
            .ok_or("declared external effect did not emit its installed typed payload")?;
        if matching.next().is_some() {
            return Err("declared external effect emitted its installed payload more than once");
        }
        let projection = emission
            .external_payload()
            .ok_or("declared external effect used an ordinary non-external emission")?;
        if projection.maximum_bytes != *maximum_payload_bytes
            || u64::try_from(projection.bytes.len()).map_err(|_| "external payload is too large")?
                > *maximum_payload_bytes
        {
            return Err("external payload projection drifted from its installed bound");
        }
        Ok(Some(projection.bytes.to_vec()))
    }
}

mod court_doors_for_test;
