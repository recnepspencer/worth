//! Prebuild component measurements from the same owner as candidate debits.

use super::DecisionReader;
use crate::domain_computation::primary_graph::application_attempt::candidate_retained_representation as representation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationEntityKey,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    ApplicationSchema, DeclaredApplicationFieldValue, OperationCreates, OperationLinks,
    OperationWrites, RequiredApplicationFieldValue, WritableCapability,
};

pub(super) fn checked(bytes: Option<usize>) -> Result<usize, Denial> {
    bytes.ok_or_else(|| {
        Denial::new(
            Kind::CandidateReservationExceeded,
            "candidate representation quote overflow",
        )
    })
}

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Retained create/key metadata for this exact key, moved unchanged to the writer.
    /// This measures logical candidate representation, not a physical memory ticket.
    pub fn candidate_create_bytes<Entity>(
        &self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: &WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<usize, Denial>
    where
        Entity: OperationCreates<Binding::Operation>,
    {
        checked(representation::created_entity_bytes(
            key.owned_allocation_capacity_bytes(),
            key.as_str().len(),
            entity.name(),
        ))
    }

    /// One newly initialized field entry, including its actual installed locator.
    /// Supply the actual encoded value's retained heap bytes, not its wire width
    /// unless that binding guarantees they are identical. Count each new entry once.
    pub fn candidate_initial_field_bytes<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        encoded_value_bytes: usize,
    ) -> Result<usize, Denial>
    where
        Entity: OperationCreates<Binding::Operation>,
        Field: OperationWrites<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.reader.candidate_field_bytes(
            field.entity(),
            field.aspect(),
            field.field(),
            encoded_value_bytes,
            false,
        )
    }

    /// One new field entry on an existing target. Includes the installed optional
    /// contract for statically optional fields. Add `candidate_update_bytes` once
    /// per target effect. A required entry stored in an optional patch also needs
    /// `candidate_field_promotion_bytes`, regardless of write order. Replacement
    /// credit stays with the writer; this quote does not simulate a write sequence.
    pub fn candidate_write_field_bytes<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        encoded_value_bytes: usize,
    ) -> Result<usize, Denial>
    where
        Field: OperationWrites<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        self.reader.candidate_field_bytes(
            field.entity(),
            field.aspect(),
            field.field(),
            encoded_value_bytes,
            true,
        )
    }

    /// Entity-name backing retained once by a new update/optional-patch effect.
    pub fn candidate_update_bytes<Entity>(
        &self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> usize {
        entity.name().len()
    }

    /// Contract backing for a required-field entry stored in an optional patch.
    /// Add once to its base field quote, whether an earlier required write is
    /// promoted by a later optional write or the required entry is inserted after
    /// the optional patch exists. Replacing that entry adds no second contract.
    /// This descriptive component neither grants authority nor reorders writes.
    pub fn candidate_field_promotion_bytes<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<usize, Denial>
    where
        Field: OperationWrites<Binding::Operation> + RequiredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        self.reader
            .candidate_field_promotion_bytes(field.entity(), field.aspect(), field.field())
    }

    /// One link with this exact owned key. `Some(key)` denotes an endpoint created
    /// by this candidate; `None` denotes an existing endpoint. These descriptions
    /// grant no target authority. Move the measured link key unchanged to the writer.
    pub fn candidate_link_bytes<Relation, From, To>(
        &self,
        _relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: &String,
        from_created: Option<&WorthQueryApplicationEntityKey<Schema, From>>,
        to_created: Option<&WorthQueryApplicationEntityKey<Schema, To>>,
    ) -> Result<usize, Denial>
    where
        Relation: OperationLinks<Binding::Operation>,
    {
        checked(representation::relation_bytes(
            key,
            from_created.map_or(0, |key| key.as_str().len()),
            to_created.map_or(0, |key| key.as_str().len()),
        ))
    }
}
