//! A keyed lookup that found nothing, or a keyed write whose key another
//! entity already holds, preserved as typed evidence through handler execution.
use worth_foundational::facade::AspectValue;

mod field_label;

/// What a keyed lookup or keyed write found.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WorthQueryKeyedEntityOutcome {
    /// No entity holds the key.
    NotFound,
    /// Another entity already holds the key, or the write names it twice.
    AlreadyExists,
}

/// Which entity and key a keyed refusal names.
///
/// Carried by the denial that refused the request, so an application branches
/// on [`Self::outcome`] and reports [`Self::entity`], [`Self::field`] and
/// [`Self::key`] instead of parsing a diagnostic subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryKeyedEntityRefusal {
    outcome: WorthQueryKeyedEntityOutcome,
    entity: String,
    field: String,
    key: Option<AspectValue>,
}

impl WorthQueryKeyedEntityRefusal {
    /// No `entity` holds `key` in its identity `field`.
    pub fn not_found(
        entity: impl Into<String>,
        field: impl Into<String>,
        key: Option<AspectValue>,
    ) -> Self {
        Self::new(WorthQueryKeyedEntityOutcome::NotFound, entity, field, key)
    }

    /// Another `entity` already holds `key` in its unique `field`.
    pub fn already_exists(
        entity: impl Into<String>,
        field: impl Into<String>,
        key: Option<AspectValue>,
    ) -> Self {
        Self::new(
            WorthQueryKeyedEntityOutcome::AlreadyExists,
            entity,
            field,
            key,
        )
    }

    fn new(
        outcome: WorthQueryKeyedEntityOutcome,
        entity: impl Into<String>,
        field: impl Into<String>,
        key: Option<AspectValue>,
    ) -> Self {
        Self {
            outcome,
            entity: entity.into(),
            field: field.into(),
            key,
        }
    }

    pub const fn outcome(&self) -> WorthQueryKeyedEntityOutcome {
        self.outcome
    }

    /// The declared entity type name.
    pub fn entity(&self) -> &str {
        &self.entity
    }

    /// The declared identity or unique field name.
    pub fn field(&self) -> &str {
        &self.field
    }

    /// The keyed field as lowercase words naming its entity exactly once, for
    /// messages: `Order` / `OrderOrderId` and `Order` / `OrderId` both read
    /// `order id`, `Item` / `Sku` reads `item sku`. [`Self::field`] stays the
    /// declared name.
    pub fn field_label(&self) -> String {
        field_label::qualified_field_label(&self.entity, &self.field)
    }

    /// The key value, when it could be encoded.
    pub const fn key(&self) -> Option<&AspectValue> {
        self.key.as_ref()
    }
}

impl std::fmt::Display for WorthQueryKeyedEntityRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found = match self.outcome {
            WorthQueryKeyedEntityOutcome::NotFound => "no entity holds",
            WorthQueryKeyedEntityOutcome::AlreadyExists => "another entity already holds",
        };
        write!(formatter, "{} {found} {}", self.entity, self.field)?;
        match &self.key {
            Some(key) => write!(formatter, " = {key:?}"),
            None => Ok(()),
        }
    }
}

impl std::error::Error for WorthQueryKeyedEntityRefusal {}
