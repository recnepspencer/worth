use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityMarkerIdentity,
    ApplicationEntityRef, ApplicationSchema,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

use super::super::{
    ApplicationQueryBinding, ApplicationQueryBindingReference, ApplicationQueryCanonicalArtifact,
    ApplicationQueryDisclosureContract, ApplicationQueryResultRelationCardinality,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversal,
};
use super::{
    ApplicationHistoryAspectSelection, ApplicationHistoryQueryBinding, ApplicationHistoryTarget,
};

mod validation;

type SubjectQuery<Schema, History> =
    <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<
        Schema,
    >>::Query;

/// Portable history meaning. It carries neither a native subject nor a retained root.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableApplicationHistoryQueryParts {
    pub name: String,
    pub history_type: WorthQueryPortableTypeIdentity,
    pub subject_read: ApplicationQueryBindingReference,
    pub target: ApplicationHistoryTarget,
    pub aspects: Vec<ApplicationHistoryAspectSelection>,
    pub metadata_disclosure: ApplicationQueryDisclosureContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ErasedApplicationHistoryQueryDefinition {
    parts: WorthQueryPortableApplicationHistoryQueryParts,
    canonical: ApplicationQueryCanonicalArtifact,
}

impl ErasedApplicationHistoryQueryDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationHistoryQueryParts) -> Self {
        let canonical = super::super::canonical_basis::prepare_history_basis(&parts);
        Self { parts, canonical }
    }

    pub fn parts(&self) -> &WorthQueryPortableApplicationHistoryQueryParts {
        &self.parts
    }

    pub fn canonical_basis(&self) -> &ApplicationQueryCanonicalArtifact {
        &self.canonical
    }

    pub fn validate(&self) -> Result<(), ApplicationHistoryQueryDefinitionDenial> {
        validation::validate(&self.parts)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHistoryQueryDefinitionDenial {
    InvalidIdentity,
    InvalidTarget,
    EmptyAspectSelection,
    InvalidAspect,
    DuplicateAspect,
    NonCanonicalAspectSelection,
    InvalidMetadataDisclosure,
}

pub struct ApplicationHistoryQueryDefinition<Schema, History> {
    erased: ErasedApplicationHistoryQueryDefinition,
    _history: PhantomData<fn() -> (Schema, History)>,
}

pub struct ApplicationHistoryQueryDefinitionBuilder<Schema, History, Record> {
    parts: WorthQueryPortableApplicationHistoryQueryParts,
    _history: PhantomData<fn() -> (Schema, History, Record)>,
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>
    ApplicationHistoryQueryDefinition<Schema, History>
{
    /// The metadata contract is mandatory and separate from SubjectRead's payload contract.
    pub fn entity<Entity: ApplicationEntityMarkerIdentity<Schema>>(
        entity: ApplicationEntityRef<Schema, Entity>,
        metadata_disclosure: ApplicationQueryDisclosureContract,
    ) -> ApplicationHistoryQueryDefinitionBuilder<Schema, History, Entity> {
        Self::builder(
            ApplicationHistoryTarget::Entity {
                entity: entity.name().to_owned(),
            },
            metadata_disclosure,
        )
    }

    /// Select actual physical occurrences of an exact declared SubjectRead relation slot.
    pub fn relation<Slot, Relation, From, To, Direction, Cardinality>(
        relation: ApplicationQueryResultRelationRef<
            SubjectQuery<Schema, History>,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
        metadata_disclosure: ApplicationQueryDisclosureContract,
    ) -> ApplicationHistoryQueryDefinitionBuilder<Schema, History, Relation>
    where
        Slot: WorthQueryPortableType,
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
    {
        Self::builder(
            ApplicationHistoryTarget::Relation {
                query: relation.slot_key().query_identity(),
                slot: relation.slot_key().slot_identity(),
                relation: relation.relation().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                direction: relation.direction(),
                cardinality: relation.cardinality(),
                output_name: relation.output_name().to_owned(),
            },
            metadata_disclosure,
        )
    }

    fn builder<Record>(
        target: ApplicationHistoryTarget,
        metadata_disclosure: ApplicationQueryDisclosureContract,
    ) -> ApplicationHistoryQueryDefinitionBuilder<Schema, History, Record> {
        ApplicationHistoryQueryDefinitionBuilder {
            parts: WorthQueryPortableApplicationHistoryQueryParts {
                name: History::IDENTITY.to_owned(),
                history_type: History::PORTABLE_TYPE_IDENTITY,
                subject_read: ApplicationQueryBindingReference::of::<Schema, History::SubjectRead>(
                ),
                target,
                aspects: Vec::new(),
                metadata_disclosure,
            },
            _history: PhantomData,
        }
    }

    pub fn into_erased(self) -> ErasedApplicationHistoryQueryDefinition {
        self.erased
    }
}

impl<Schema, History, Record> ApplicationHistoryQueryDefinitionBuilder<Schema, History, Record> {
    pub fn aspect<Aspect: ApplicationAspectMarkerIdentity<Schema, Record>>(
        mut self,
        aspect: ApplicationAspectRef<Schema, Record, Aspect>,
    ) -> Self {
        self.parts.aspects.push(ApplicationHistoryAspectSelection {
            aspect: aspect.name().to_owned(),
            identity: aspect.identity(),
            revision: aspect.revision(),
        });
        self
    }

    pub fn finish(
        mut self,
    ) -> Result<
        ApplicationHistoryQueryDefinition<Schema, History>,
        ApplicationHistoryQueryDefinitionDenial,
    > {
        self.parts.aspects.sort();
        let erased = ErasedApplicationHistoryQueryDefinition::from_untrusted_parts(self.parts);
        erased.validate()?;
        Ok(ApplicationHistoryQueryDefinition {
            erased,
            _history: PhantomData,
        })
    }
}
