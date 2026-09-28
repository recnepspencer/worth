//! Retained and external emissions admitted into one effect program.

use std::any::{Any, TypeId};
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationRetainedEffectBinding,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationEmission {
    effect: &'static str,
    payload_type: WorthQueryPortableTypeIdentity,
    payload_type_id: TypeId,
    payload: Arc<dyn Any + Send + Sync>,
    retained_bytes: u64,
    measure_retained_bytes: fn(&(dyn Any + Send + Sync)) -> Option<u64>,
    external_payload: Option<WorthQueryExternalPayloadProjection>,
}

#[derive(Clone)]
struct WorthQueryExternalPayloadProjection {
    bytes: Arc<[u8]>,
    maximum_bytes: u64,
}

impl WorthQueryApplicationEmission {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn new<Binding>(
        effect: &'static str,
        payload: Binding::Value,
    ) -> Self
    where
        Binding: ApplicationRetainedEffectBinding,
        Binding::Value: Send + Sync,
    {
        let retained_bytes = Binding::retained_bytes(&payload);
        Self {
            effect,
            payload_type: Binding::IDENTITY,
            payload_type_id: TypeId::of::<Binding::Value>(),
            payload: Arc::new(payload),
            retained_bytes,
            measure_retained_bytes: measure_retained_bytes::<Binding>,
            external_payload: None,
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
        let bytes = Binding::external_effect_bytes(&payload);
        let encoded_len = u64::try_from(bytes.len()).map_err(|_| ())?;
        if Binding::MAX_EXTERNAL_BYTES == 0 || encoded_len > Binding::MAX_EXTERNAL_BYTES {
            return Err(());
        }
        let mut emission = Self::new::<Binding>(effect, payload);
        emission.external_payload = Some(WorthQueryExternalPayloadProjection {
            bytes: bytes.into(),
            maximum_bytes: Binding::MAX_EXTERNAL_BYTES,
        });
        Ok(emission)
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_lifecycle(
        derived: &worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect,
    ) -> Self {
        Self {
            effect: derived.effect(),
            payload_type: derived.payload_identity(),
            payload_type_id: derived.payload_type_id(),
            payload: derived.payload(),
            retained_bytes: derived.retained_bytes(),
            measure_retained_bytes: derived.measure_retained_bytes(),
            external_payload: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn is_exact_lifecycle(
        &self,
        derived: &worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect,
    ) -> bool {
        self.effect == derived.effect()
            && self.payload_type == derived.payload_identity()
            && self.payload_type_id == derived.payload_type_id()
            && self.retained_bytes == derived.retained_bytes()
            && derived.payload_is(&self.payload)
            && self.is_well_formed()
    }

    pub(in crate::domain_computation::primary_graph) fn is_well_formed(&self) -> bool {
        !self.effect.is_empty()
            && self.payload_type.is_valid()
            && self.payload_type_id == self.payload.as_ref().type_id()
            && (self.measure_retained_bytes)(self.payload.as_ref()) == Some(self.retained_bytes)
    }

    pub(in crate::domain_computation::primary_graph) const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn candidate_retained_representation_bytes(
        &self,
    ) -> Option<usize> {
        usize::try_from(self.retained_bytes).ok()?.checked_add(
            self.external_payload
                .as_ref()
                .map_or(0, |payload| payload.bytes.len()),
        )
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

    #[cfg(test)]
    pub(crate) const fn effect(&self) -> &'static str {
        self.effect
    }

    #[cfg(test)]
    pub(crate) fn payload<Payload: 'static>(&self) -> Option<&Payload> {
        self.payload.downcast_ref()
    }
}

fn measure_retained_bytes<Binding>(payload: &(dyn Any + Send + Sync)) -> Option<u64>
where
    Binding: ApplicationRetainedEffectBinding,
{
    payload
        .downcast_ref::<Binding::Value>()
        .map(Binding::retained_bytes)
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryAdmittedApplicationEmissionBatch {
    emissions: Vec<WorthQueryApplicationEmission>,
    retained_bytes: u64,
}

impl WorthQueryAdmittedApplicationEmissionBatch {
    pub(in crate::domain_computation::primary_graph) fn admit(
        emissions: Vec<WorthQueryApplicationEmission>,
        retained_bytes_ceiling: u64,
    ) -> Result<Self, &'static str> {
        let retained_bytes = emissions.iter().try_fold(0_u64, |total, emission| {
            if !emission.is_well_formed() {
                return Err("application commit carried an invalid typed emission");
            }
            total
                .checked_add(emission.retained_bytes())
                .ok_or("application emission retained-byte count overflowed")
        })?;
        if retained_bytes > retained_bytes_ceiling {
            return Err("application emission exceeded installed retained-byte ceiling");
        }
        Ok(Self {
            emissions,
            retained_bytes,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (Vec<WorthQueryApplicationEmission>, u64) {
        (self.emissions, self.retained_bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.emissions.len()
    }

    pub(in crate::domain_computation::primary_graph) const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
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
