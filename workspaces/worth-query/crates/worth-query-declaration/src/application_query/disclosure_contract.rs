use worth_foundational::facade::{AspectMask, AspectValue, CanonicalFieldPath, FieldKey};

use super::{
    ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationCardinality, ApplicationQueryResultRelationRef,
    ApplicationQueryResultTraversal,
};
use crate::application_capability::ApplicationCapabilityRef;
use crate::application_schema::{ApplicationEncodedScalarValue, ApplicationScalarValueBinding};

mod history;
pub use history::ApplicationHistoryMetadataDisclosureDenial;
mod inspection;

mod classification;
mod components;
mod influence;
mod optional_aspect;
mod resource;
mod selector;
pub use classification::ApplicationDisclosureClassification;
pub use influence::{ApplicationQueryInfluenceContract, ApplicationQueryObservableInfluence};
pub use selector::ApplicationQueryDisclosureSelector;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryDisclosurePosture {
    Public,
    InstalledPolicyRequired,
    Governed,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryDisclosureRule {
    selector: ApplicationQueryDisclosureSelector,
    disclosure_value: AspectValue,
    influence: ApplicationQueryInfluenceContract,
}

impl ApplicationQueryDisclosureRule {
    pub fn from_untrusted_fields(
        selector: ApplicationQueryDisclosureSelector,
        disclosure_value: AspectValue,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self {
        Self {
            selector,
            disclosure_value,
            influence,
        }
    }

    pub const fn selector(&self) -> &ApplicationQueryDisclosureSelector {
        &self.selector
    }

    pub const fn disclosure_value(&self) -> &AspectValue {
        &self.disclosure_value
    }

    pub const fn influence(&self) -> &ApplicationQueryInfluenceContract {
        &self.influence
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryDisclosureContract {
    posture: ApplicationQueryDisclosurePosture,
    classification: String,
    capability_name: Option<String>,
    capability_type: Option<crate::portable_identity::WorthQueryPortableTypeIdentity>,
    rules: Vec<ApplicationQueryDisclosureRule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryDisclosureParts {
    pub posture: ApplicationQueryDisclosurePosture,
    pub classification: String,
    pub capability_name: Option<String>,
    pub capability_type: Option<crate::portable_identity::WorthQueryPortableTypeIdentity>,
    pub rules: Vec<ApplicationQueryDisclosureRule>,
}

impl ApplicationQueryDisclosureContract {
    pub(crate) fn canonical_basis(&self) -> super::ApplicationQueryCanonicalArtifact {
        super::canonical_basis::prepare_disclosure_basis(self)
    }
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryDisclosureParts) -> Self {
        Self {
            posture: parts.posture,
            classification: parts.classification,
            capability_name: parts.capability_name,
            capability_type: parts.capability_type,
            rules: parts.rules,
        }
    }

    /// Govern the whole orbit's identity, length, membership and successor order.
    pub fn orbit_by<
        Schema,
        Query,
        Slot,
        Relation,
        Entity,
        DisclosureBinding: ApplicationScalarValueBinding,
    >(
        mut self,
        orbit: super::ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Entity>,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Orbit(orbit.into_meaning()),
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
    pub fn public() -> Self {
        Self {
            posture: ApplicationQueryDisclosurePosture::Public,
            classification: "public".to_owned(),
            capability_name: None,
            capability_type: None,
            rules: Vec::new(),
        }
    }

    /// Disclosure that requires an installed policy for `Classification`.
    pub fn installed_policy<Classification: ApplicationDisclosureClassification>(
        _: Classification,
    ) -> Self {
        Self {
            posture: ApplicationQueryDisclosurePosture::InstalledPolicyRequired,
            classification: Classification::CLASSIFICATION.to_owned(),
            capability_name: None,
            capability_type: None,
            rules: Vec::new(),
        }
    }

    /// Disclosure governed by `capability` under the typed `Classification`.
    pub fn governed_by<Classification, Schema, Capability>(
        _: Classification,
        capability: ApplicationCapabilityRef<Schema, Capability>,
    ) -> Self
    where
        Classification: ApplicationDisclosureClassification,
    {
        Self {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: Classification::CLASSIFICATION.to_owned(),
            capability_name: Some(capability.name().to_owned()),
            capability_type: Some(capability.marker_identity()),
            rules: Vec::new(),
        }
    }

    pub fn use_field_by<Field, DisclosureBinding>(
        mut self,
        field: Field,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Result<Self, crate::application_schema::ApplicationSchemaDeclarationDenial>
    where
        Field: crate::application_schema::fragment::RecordSelection,
        DisclosureBinding: ApplicationScalarValueBinding,
    {
        field.checked()?;
        let field_key = FieldKey::new(field.field())
            .expect("typed application fields are valid Foundational keys");
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::InternalField {
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
                projection_mask: AspectMask::new([CanonicalFieldPath::single(field_key.clone())]),
                diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(field_key)]),
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        Ok(self)
    }

    /// Governs internal traversal of one declared relation (for example in a
    /// related-record absence/existence predicate) without projecting it in the
    /// result shape.
    pub fn use_relation_by<Schema, Relation, From, To, DisclosureBinding>(
        mut self,
        relation: crate::application_schema::ApplicationRelationRef<Schema, Relation, From, To>,
        direction: super::RelationDirection,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        DisclosureBinding: ApplicationScalarValueBinding,
    {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::InternalRelation {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                direction,
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn disclose_field_by<Query, Slot, Schema, Entity, Aspect, Field, Value, DisclosureBinding>(
        mut self,
        selector: ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        DisclosureBinding: ApplicationScalarValueBinding,
        Query: super::ApplicationQueryMarkerIdentity<Schema> + 'static,
        Slot: crate::portable_identity::WorthQueryPortableType + 'static,
    {
        let field = FieldKey::new(selector.field())
            .expect("typed application-query fields are valid Foundational keys");
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Field {
                query_type: selector.slot_key().query_identity(),
                slot_type: selector.slot_key().slot_identity(),
                entity: selector.entity().to_owned(),
                aspect: selector.aspect().to_owned(),
                field: selector.field().to_owned(),
                output_name: selector.output_name().to_owned(),
                scalar_family: selector.scalar_family(),
                value_type: Field::Binding::IDENTITY,
                presence: crate::application_query::ApplicationQueryResultFieldPresence::Required,
                projection_mask: AspectMask::new([CanonicalFieldPath::single(field.clone())]),
                diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(field)]),
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn disclose_optional_field_by<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        DisclosureBinding,
    >(
        mut self,
        selector: ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        DisclosureBinding: ApplicationScalarValueBinding,
        Query: super::ApplicationQueryMarkerIdentity<Schema> + 'static,
        Slot: crate::portable_identity::WorthQueryPortableType + 'static,
    {
        let field = FieldKey::new(selector.field())
            .expect("typed application-query fields are valid Foundational keys");
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Field {
                query_type: selector.slot_key().query_identity(),
                slot_type: selector.slot_key().slot_identity(),
                entity: selector.entity().to_owned(),
                aspect: selector.aspect().to_owned(),
                field: selector.field().to_owned(),
                output_name: selector.output_name().to_owned(),
                scalar_family: selector.scalar_family(),
                value_type: Field::Binding::IDENTITY,
                presence: crate::application_query::ApplicationQueryResultFieldPresence::Optional,
                projection_mask: AspectMask::new([CanonicalFieldPath::single(field.clone())]),
                diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(field)]),
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn disclose_relation_by<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        Direction,
        Cardinality,
        DisclosureBinding,
    >(
        mut self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self
    where
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
        DisclosureBinding: ApplicationScalarValueBinding,
        Query: super::ApplicationQueryMarkerIdentity<Schema> + 'static,
        Slot: crate::portable_identity::WorthQueryPortableType + 'static,
    {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Relation {
                query_type: selector.slot_key().query_identity(),
                slot_type: selector.slot_key().slot_identity(),
                relation: selector.relation().to_owned(),
                from: selector.from().to_owned(),
                to: selector.to().to_owned(),
                direction: selector.direction(),
                cardinality: selector.cardinality(),
                output_name: selector.output_name().to_owned(),
            },
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }

    pub const fn posture(&self) -> ApplicationQueryDisclosurePosture {
        self.posture
    }

    pub fn classification(&self) -> &str {
        &self.classification
    }

    pub fn capability_name(&self) -> Option<&str> {
        self.capability_name.as_deref()
    }

    pub fn capability_type(&self) -> Option<&str> {
        self.capability_type
            .as_ref()
            .map(|identity| identity.as_str())
    }

    pub fn capability_identity(
        &self,
    ) -> Option<crate::portable_identity::WorthQueryPortableTypeIdentity> {
        self.capability_type.clone()
    }

    pub fn rules(&self) -> &[ApplicationQueryDisclosureRule] {
        &self.rules
    }
}
