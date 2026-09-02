use std::collections::BTreeMap;
use std::sync::Arc;

use crate::capability::{
    UiIntentBoolean, UiIntentPayloadFieldKind, UiIntentText, UiIntentUnsigned64,
};
use crate::declaration::{
    UiIntentApplicationFact, UiIntentApplicationFactPlan, UiIntentApplicationFactValue,
};

#[path = "application_fact_state/validation.rs"]
mod validation;
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use validation::{
    UiAdmittedValidationAppearanceTarget, UiValidationAppearanceFactDenial,
};
pub(crate) use validation::{UiValidationAppearanceClass, UiValidationAppearanceFactSnapshot};

pub(crate) struct UiIntentApplicationFactState {
    slots_by_identity: BTreeMap<Arc<str>, crate::declaration::UiIntentApplicationFactSlot>,
    facts: Box<[UiIntentApplicationFactRecord]>,
    validation_owner: Option<validation::UiValidationAppearanceOwner>,
}

struct UiIntentApplicationFactRecord {
    identity: Arc<str>,
    revision: u64,
    text_byte_budget: usize,
    value: UiIntentApplicationFactValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentApplicationFactUpdateReceipt {
    identity: Arc<str>,
    revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentApplicationFactUpdateDenial {
    UnknownFact {
        identity: Box<str>,
    },
    FactKindMismatch {
        identity: Box<str>,
        registered: UiIntentPayloadFieldKind,
        submitted: UiIntentPayloadFieldKind,
    },
    TextBudgetExceeded {
        identity: Box<str>,
        observed: usize,
        maximum: usize,
    },
    RevisionExhausted {
        identity: Box<str>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiIntentApplicationInputRevision {
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    identity: Arc<str>,
    revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentApplicationInputReference {
    Text {
        revision: UiIntentApplicationInputRevision,
        value: Arc<str>,
    },
    Boolean {
        revision: UiIntentApplicationInputRevision,
        value: bool,
    },
    Unsigned64 {
        revision: UiIntentApplicationInputRevision,
        value: u64,
    },
}

impl UiIntentApplicationFactState {
    pub(crate) fn activate(
        plan: &UiIntentApplicationFactPlan,
        validation_appearance_enabled: bool,
    ) -> Self {
        let mut slots_by_identity = BTreeMap::new();
        let mut facts = (0..plan.entries().len())
            .map(|_| None)
            .collect::<Vec<Option<UiIntentApplicationFactRecord>>>();
        for (identity, definition) in plan.entries() {
            slots_by_identity.insert(Arc::clone(identity), definition.slot());
            facts[definition.slot().index()] = Some(UiIntentApplicationFactRecord {
                identity: Arc::clone(identity),
                revision: 1,
                text_byte_budget: definition.text_byte_budget(),
                value: definition.initial().clone(),
            });
        }
        Self {
            slots_by_identity,
            facts: facts
                .into_iter()
                .map(|record| record.expect("application fact slots are dense"))
                .collect(),
            validation_owner: validation_appearance_enabled
                .then(validation::UiValidationAppearanceOwner::new),
        }
    }

    pub(crate) fn reconcile_validation_appearance(&mut self, enabled: bool) {
        match (enabled, self.validation_owner.is_some()) {
            (true, false) => {
                self.validation_owner = Some(validation::UiValidationAppearanceOwner::new());
            }
            (false, true) => self.validation_owner = None,
            _ => {}
        }
    }

    pub(crate) fn update_text(
        &mut self,
        fact: &UiIntentApplicationFact<UiIntentText>,
        value: impl Into<Arc<str>>,
    ) -> Result<UiIntentApplicationFactUpdateReceipt, UiIntentApplicationFactUpdateDenial> {
        let value = value.into();
        let record = self.require(fact.identity(), UiIntentPayloadFieldKind::Text)?;
        if value.len() > record.text_byte_budget {
            return Err(UiIntentApplicationFactUpdateDenial::TextBudgetExceeded {
                identity: fact.identity().into(),
                observed: value.len(),
                maximum: record.text_byte_budget,
            });
        }
        update_record(
            fact.identity(),
            record,
            UiIntentApplicationFactValue::Text(value),
        )
    }

    pub(crate) fn update_boolean(
        &mut self,
        fact: &UiIntentApplicationFact<UiIntentBoolean>,
        value: bool,
    ) -> Result<UiIntentApplicationFactUpdateReceipt, UiIntentApplicationFactUpdateDenial> {
        let record = self.require(fact.identity(), UiIntentPayloadFieldKind::Boolean)?;
        update_record(
            fact.identity(),
            record,
            UiIntentApplicationFactValue::Boolean(value),
        )
    }

    pub(crate) fn update_unsigned64(
        &mut self,
        fact: &UiIntentApplicationFact<UiIntentUnsigned64>,
        value: u64,
    ) -> Result<UiIntentApplicationFactUpdateReceipt, UiIntentApplicationFactUpdateDenial> {
        let record = self.require(fact.identity(), UiIntentPayloadFieldKind::Unsigned64)?;
        update_record(
            fact.identity(),
            record,
            UiIntentApplicationFactValue::Unsigned64(value),
        )
    }

    pub(crate) fn input_reference(
        &self,
        slot: crate::declaration::UiIntentApplicationFactSlot,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Option<UiIntentApplicationInputReference> {
        let record = self.facts.get(slot.index())?;
        let revision = UiIntentApplicationInputRevision {
            generation: generation.clone(),
            identity: Arc::clone(&record.identity),
            revision: record.revision,
        };
        Some(match &record.value {
            UiIntentApplicationFactValue::Text(value) => UiIntentApplicationInputReference::Text {
                revision,
                value: Arc::clone(value),
            },
            UiIntentApplicationFactValue::Boolean(value) => {
                UiIntentApplicationInputReference::Boolean {
                    revision,
                    value: *value,
                }
            }
            UiIntentApplicationFactValue::Unsigned64(value) => {
                UiIntentApplicationInputReference::Unsigned64 {
                    revision,
                    value: *value,
                }
            }
        })
    }

    pub(crate) fn is_current_reference(
        &self,
        expected: &UiIntentApplicationInputReference,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        let Some(slot) = self
            .slots_by_identity
            .get(expected.revision().identity())
            .copied()
        else {
            return false;
        };
        self.input_reference(slot, generation).as_ref() == Some(expected)
    }

    fn require(
        &mut self,
        identity: &str,
        submitted: UiIntentPayloadFieldKind,
    ) -> Result<&mut UiIntentApplicationFactRecord, UiIntentApplicationFactUpdateDenial> {
        let slot = self
            .slots_by_identity
            .get(identity)
            .copied()
            .ok_or_else(|| UiIntentApplicationFactUpdateDenial::UnknownFact {
                identity: identity.into(),
            })?;
        let record = self
            .facts
            .get_mut(slot.index())
            .expect("registered application fact slot is present");
        let registered = value_kind(&record.value);
        if registered != submitted {
            return Err(UiIntentApplicationFactUpdateDenial::FactKindMismatch {
                identity: identity.into(),
                registered,
                submitted,
            });
        }
        Ok(record)
    }
}

fn update_record(
    identity: &str,
    record: &mut UiIntentApplicationFactRecord,
    value: UiIntentApplicationFactValue,
) -> Result<UiIntentApplicationFactUpdateReceipt, UiIntentApplicationFactUpdateDenial> {
    record.revision = record.revision.checked_add(1).ok_or_else(|| {
        UiIntentApplicationFactUpdateDenial::RevisionExhausted {
            identity: identity.into(),
        }
    })?;
    record.value = value;
    Ok(UiIntentApplicationFactUpdateReceipt {
        identity: identity.into(),
        revision: record.revision,
    })
}

fn value_kind(value: &UiIntentApplicationFactValue) -> UiIntentPayloadFieldKind {
    match value {
        UiIntentApplicationFactValue::Text(_) => UiIntentPayloadFieldKind::Text,
        UiIntentApplicationFactValue::Boolean(_) => UiIntentPayloadFieldKind::Boolean,
        UiIntentApplicationFactValue::Unsigned64(_) => UiIntentPayloadFieldKind::Unsigned64,
    }
}

impl UiIntentApplicationFactUpdateReceipt {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

impl UiIntentApplicationInputReference {
    pub(crate) const fn revision(&self) -> &UiIntentApplicationInputRevision {
        match self {
            Self::Text { revision, .. }
            | Self::Boolean { revision, .. }
            | Self::Unsigned64 { revision, .. } => revision,
        }
    }

    pub(crate) const fn kind(&self) -> UiIntentPayloadFieldKind {
        match self {
            Self::Text { .. } => UiIntentPayloadFieldKind::Text,
            Self::Boolean { .. } => UiIntentPayloadFieldKind::Boolean,
            Self::Unsigned64 { .. } => UiIntentPayloadFieldKind::Unsigned64,
        }
    }

    pub(crate) fn text_value(&self) -> Option<Arc<str>> {
        match self {
            Self::Text { value, .. } => Some(Arc::clone(value)),
            Self::Boolean { .. } | Self::Unsigned64 { .. } => None,
        }
    }

    pub(crate) const fn boolean_value(&self) -> Option<bool> {
        match self {
            Self::Boolean { value, .. } => Some(*value),
            Self::Text { .. } | Self::Unsigned64 { .. } => None,
        }
    }

    pub(crate) const fn unsigned64_value(&self) -> Option<u64> {
        match self {
            Self::Unsigned64 { value, .. } => Some(*value),
            Self::Text { .. } | Self::Boolean { .. } => None,
        }
    }
}

impl UiIntentApplicationInputRevision {
    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
}
