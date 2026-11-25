//! The `on_fire` hook of a stock recurring rule: a typed domain action that
//! runs atomically inside each fire commit, after the rule's `fired_count`
//! advances, optionally in a target entity the fire resolves by a unique key.
use super::recurring::WorthQueryRecurringOccurrence;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationEntityKey,
    WorthQueryApplicationEntityKeyDenial,
};

/// One fire of a recurring rule, as its `on_fire` hook sees it.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryRecurringFire<'fire> {
    occurrence: &'fire WorthQueryRecurringOccurrence,
}

impl<'fire> WorthQueryRecurringFire<'fire> {
    #[doc(hidden)]
    pub fn new(occurrence: &'fire WorthQueryRecurringOccurrence) -> Self {
        Self { occurrence }
    }

    /// The firing rule's identifier.
    pub fn rule_id(&self) -> &'fire str {
        &self.occurrence.rule_id
    }

    /// This fire's 1-based number: the rule's `fired_count` once it commits.
    pub fn fire_number(&self) -> u64 {
        self.occurrence.fired_count.saturating_add(1)
    }

    /// The due instant, in clock nanoseconds, of the occurrence firing now.
    pub fn due_ns(&self) -> u64 {
        self.occurrence.due_ns
    }

    /// The rule's interval between occurrences, in clock nanoseconds.
    pub fn interval_ns(&self) -> u64 {
        self.occurrence.interval_ns
    }

    /// A key determined by `(rule_id, fire_number)` alone, so an entity the
    /// hook creates under it is created once per fire.
    pub fn entity_key<Schema, Entity>(
        &self,
    ) -> Result<WorthQueryApplicationEntityKey<Schema, Entity>, WorthQueryRecurringFireDenial> {
        WorthQueryApplicationEntityKey::new(format!(
            "recurrence:{}:fire:{}",
            self.rule_id(),
            self.fire_number()
        ))
        .map_err(WorthQueryRecurringFireDenial::EntityKey)
    }
}

/// Why an `on_fire` hook refused; the whole fire, including its
/// `fired_count` advance, commits nothing.
#[derive(Debug)]
pub enum WorthQueryRecurringFireDenial {
    /// The deterministic fire key was not a valid entity key.
    EntityKey(WorthQueryApplicationEntityKeyDenial),
    /// An effect the hook authored was refused.
    Effect(WorthQueryApplicationAttemptDenial),
}

impl From<WorthQueryApplicationAttemptDenial> for WorthQueryRecurringFireDenial {
    fn from(denial: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Effect(denial)
    }
}

impl std::fmt::Display for WorthQueryRecurringFireDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityKey(denial) => write!(formatter, "recurring fire key refused: {denial:?}"),
            Self::Effect(denial) => write!(formatter, "recurring fire effect refused: {denial:?}"),
        }
    }
}

impl std::error::Error for WorthQueryRecurringFireDenial {}

/// A domain action declared with `on_fire: Hook { creates: [..]; links: [..];
/// writes: [..]; }` on `worth_query_recurring_rule!`. The access block grants
/// the fire operation exactly those effects; `on_fire` authors them inside the
/// fire commit, which is applied once per occurrence however often catch-up
/// runs.
pub trait WorthQueryRecurringFireHook<Schema, Operation, RuleEntity> {
    /// Authors this fire's domain effects; `rule` is the firing rule row.
    fn on_fire(
        fire: WorthQueryRecurringFire<'_>,
        rule: &WorthQueryApplicationEffectEntity<Schema, RuleEntity>,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<
            Schema,
            Operation,
            WorthQueryRecurringOccurrence,
            RuleEntity,
        >,
    ) -> Result<(), WorthQueryRecurringFireDenial>;
}

/// A domain action declared with `on_fire: Hook { creates: [..]; links: [..];
/// writes: [..]; } in Target by TargetFacts::key()` on
/// `worth_query_recurring_rule!`: each fire first resolves its `Target` row
/// whose unique `key` field equals [`Self::target_key`], at the fire's
/// decision basis, and hands it to `on_fire` beside the rule row.
///
/// The fire reads the target's key field implicitly; a target that does not
/// resolve refuses the whole fire, which commits nothing.
pub trait WorthQueryRecurringTargetedFireHook<Schema, Operation, RuleEntity, Target> {
    /// The value type of the target's unique key field.
    type TargetKey;

    /// The unique key naming the entity this fire acts in.
    fn target_key(fire: WorthQueryRecurringFire<'_>) -> Self::TargetKey;

    /// Authors this fire's domain effects in `target`; `rule` is the firing
    /// rule row.
    fn on_fire(
        fire: WorthQueryRecurringFire<'_>,
        rule: &WorthQueryApplicationEffectEntity<Schema, RuleEntity>,
        target: &WorthQueryApplicationEffectEntity<Schema, Target>,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<
            Schema,
            Operation,
            WorthQueryRecurringOccurrence,
            RuleEntity,
        >,
    ) -> Result<(), WorthQueryRecurringFireDenial>;
}
