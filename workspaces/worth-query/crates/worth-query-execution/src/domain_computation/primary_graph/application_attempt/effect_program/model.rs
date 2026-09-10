use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectContractBasis};
use worth_query_declaration::facade::application_schema::{
    ApplicationEffectPayload, ApplicationExternalEffectPayload,
};
use worth_query_declaration::facade::portable_identity::{
    WorthQueryPortableType, WorthQueryPortableTypeIdentity,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::transactions::EntityReference;

use super::super::read_set::WorthQueryCompleteApplicationReadSet;
use super::super::WorthQueryProjectedApplicationMutation;

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
    pub(in crate::domain_computation::primary_graph::application_attempt) fn new<Payload>(
        effect: &'static str,
        payload: Payload,
    ) -> Self
    where
        Payload: ApplicationEffectPayload + WorthQueryPortableType,
    {
        let retained_bytes = payload.retained_bytes();
        Self {
            effect,
            payload_type: Payload::PORTABLE_TYPE_IDENTITY,
            payload_type_id: TypeId::of::<Payload>(),
            payload: Arc::new(payload),
            retained_bytes,
            measure_retained_bytes: measure_retained_bytes::<Payload>,
            external_payload: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn new_external<Payload>(
        effect: &'static str,
        payload: Payload,
    ) -> Result<Self, ()>
    where
        Payload: ApplicationExternalEffectPayload + WorthQueryPortableType,
    {
        let bytes = payload.external_effect_bytes();
        let encoded_len = u64::try_from(bytes.len()).map_err(|_| ())?;
        if Payload::MAX_EXTERNAL_BYTES == 0 || encoded_len > Payload::MAX_EXTERNAL_BYTES {
            return Err(());
        }
        let mut emission = Self::new(effect, payload);
        emission.external_payload = Some(WorthQueryExternalPayloadProjection {
            bytes: bytes.into(),
            maximum_bytes: Payload::MAX_EXTERNAL_BYTES,
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
        Payload: ApplicationEffectPayload,
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

fn measure_retained_bytes<Payload>(payload: &(dyn Any + Send + Sync)) -> Option<u64>
where
    Payload: ApplicationEffectPayload,
{
    payload
        .downcast_ref::<Payload>()
        .map(ApplicationEffectPayload::retained_bytes)
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

pub struct WorthQueryApplicationEffectEntity<Schema, Entity> {
    pub(super) reference: EntityReference,
    pub(super) entity: String,
    pub(super) created_effect: Option<usize>,
    pub(super) program: Arc<()>,
    pub(super) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryApplicationOptionalFieldWrite
{
    pub(in crate::domain_computation::primary_graph::application_attempt) contract:
        PortableAspectContractBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) value:
        Option<AspectValue>,
}

pub(in crate::domain_computation::primary_graph::application_attempt) enum WorthQueryApplicationRealizedEffect
{
    CreateEntity {
        kind: KindId,
        key: String,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
    },
    UpdateEntity {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
    },
    PatchOptionalEntityFields {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        kind: KindId,
        key: String,
        from: EntityReference,
        to: EntityReference,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
    Emit(WorthQueryApplicationEmission),
}

pub struct WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt) read_set:
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
    pub(in crate::domain_computation::primary_graph::application_attempt) effects:
        Vec<WorthQueryApplicationRealizedEffect>,
    pub(in crate::domain_computation::primary_graph::application_attempt) emission_retained_bytes:
        u64,
    pub(in crate::domain_computation::primary_graph::application_attempt) emission_retained_bytes_ceiling:
        u64,
    pub(in crate::domain_computation::primary_graph::application_attempt) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
}

pub struct WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope> {
    pub(super) read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub(super) layout: Arc<super::super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    pub(super) program: Arc<()>,
    pub(super) effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(super) keys: BTreeSet<(KindId, String)>,
    pub(super) emission_retained_bytes: u64,
    pub(super) emission_retained_bytes_ceiling: u64,
    pub(super) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn belongs_to_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.read_set
            .admission
            .belongs_to(runtime_authority, binding_identity)
    }

    pub(in crate::domain_computation::primary_graph) fn product_branch(
        &self,
    ) -> crate::basis::WorthQueryProductBranch {
        self.read_set.lease.product().product_branch()
    }
}
