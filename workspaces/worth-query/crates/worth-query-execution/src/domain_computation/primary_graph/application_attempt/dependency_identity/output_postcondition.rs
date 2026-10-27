#[cfg(test)]
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
#[cfg(test)]
use worth_relational::facade::identity::EntityId;

#[cfg(test)]
use super::super::effect_program::WorthQueryApplicationRealizedEffect;
use super::super::WorthQueryApplicationObservedFact;

/// The facts a producer's output keeps to answer whether it is current,
/// normalized to its own effect, and the ordinals among them, ascending, of
/// the facts an owner call of its partitioned computation read and that
/// effect replaced. Normalizing hides such a replacement, so the commit's
/// rebase marks those ordinals moved.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct OutputCurrentnessFacts {
    facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    moved_by_own_effect: std::sync::Arc<[usize]>,
    carry: Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::PostconditionCarry,
    >,
}

impl OutputCurrentnessFacts {
    /// Query owns these newly assembled program facts before any effect. Move
    /// them through completed-read construction, never relabel opaque storage.
    /// No computation supplied postconditions: candidate rebase must resolve
    /// every fact, without a carry or a normalized own-effect ordinal.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn admit_owned_program_facts(
        facts: Vec<WorthQueryApplicationObservedFact>,
        control: super::super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<Self, super::super::retained_decision_facts::StoreDenial> {
        use crate::domain_computation::primary_graph::application_checkpoint::facts::NativeFactPayloadBuilder;
        control.check_live()?;
        let mut builder = NativeFactPayloadBuilder::allocate_completed_read(facts.len(), control)?;
        for fact in facts {
            control.check_live()?;
            builder.push(fact)?;
        }
        control.check_live()?;
        Ok(Self {
            facts: builder.seal()?,
            carry: None,
            moved_by_own_effect: std::sync::Arc::from([]),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn rebase_carry(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::PostconditionCarry,
    > {
        self.carry.clone()
    }
    pub(super) fn with_rebase_carry(
        mut self,
        carry: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::PostconditionCarry>,
    ) -> Self {
        self.carry = carry;
        self
    }
    pub(in crate::domain_computation::primary_graph) const fn facts(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
        &self.facts
    }

    pub(in crate::domain_computation::primary_graph) fn moved_by_own_effect(
        &self,
    ) -> std::sync::Arc<[usize]> {
        std::sync::Arc::clone(&self.moved_by_own_effect)
    }
}

/// Upstream facts have passed the retained computation-current door. The
/// consumer's own effect is tracked when its completed source is rebased.
impl From<crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts>
    for OutputCurrentnessFacts
{
    fn from(
        facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
    ) -> Self {
        Self {
            facts: facts.facts().clone(),
            carry: None,
            moved_by_own_effect: std::sync::Arc::from([]),
        }
    }
}

pub(super) fn complete_content_currentness_facts(
    facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    moved: Vec<usize>,
    reusable: bool,
) -> OutputCurrentnessFacts {
    if reusable {
        OutputCurrentnessFacts {
            facts,
            carry: None,
            moved_by_own_effect: moved.into(),
        }
    } else {
        OutputCurrentnessFacts {
            facts: std::sync::Arc::<[WorthQueryApplicationObservedFact]>::from([]).into(),
            carry: None,
            moved_by_own_effect: std::sync::Arc::from([]),
        }
    }
}

#[cfg(test)]
pub(super) fn complete_output_currentness_facts(
    facts: Vec<WorthQueryApplicationObservedFact>,
    moved_by_own_effect: Vec<usize>,
) -> OutputCurrentnessFacts {
    if facts.iter().all(is_output_currentness_fact) {
        OutputCurrentnessFacts {
            facts: std::sync::Arc::<[WorthQueryApplicationObservedFact]>::from(facts).into(),
            carry: None,
            moved_by_own_effect: moved_by_own_effect.into(),
        }
    } else {
        // A partial subset would misrepresent the original producer's read
        // boundary. Empty is the explicit non-reusable marker; the complete
        // dependency digest still governs idempotency separately.
        OutputCurrentnessFacts {
            facts: std::sync::Arc::<[WorthQueryApplicationObservedFact]>::from([]).into(),
            carry: None,
            moved_by_own_effect: std::sync::Arc::from([]),
        }
    }
}

pub(super) fn is_output_currentness_fact(fact: &WorthQueryApplicationObservedFact) -> bool {
    matches!(
        fact,
        WorthQueryApplicationObservedFact::RetiredOutputEntity { .. }
            | WorthQueryApplicationObservedFact::SourcePublication { .. }
            | WorthQueryApplicationObservedFact::StructuralWalk { .. }
            | WorthQueryApplicationObservedFact::SourceEntity { .. }
            | WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
            | WorthQueryApplicationObservedFact::SourceFieldRevision { .. }
            | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            | WorthQueryApplicationObservedFact::Entity { .. }
            | WorthQueryApplicationObservedFact::Field { .. }
            | WorthQueryApplicationObservedFact::AbsentField { .. }
            | WorthQueryApplicationObservedFact::OptionalAspectAbsent { .. }
            | WorthQueryApplicationObservedFact::AspectPresence { .. }
            | WorthQueryApplicationObservedFact::Relation { .. }
            | WorthQueryApplicationObservedFact::Adjacency { .. }
            | WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
    )
}

#[cfg(test)]
pub(super) fn normalized_output_facts(
    facts: &(impl FactSequence + ?Sized),
    effects: &[WorthQueryApplicationRealizedEffect],
) -> Vec<WorthQueryApplicationObservedFact> {
    facts
        .iter()
        .map(|fact| match fact {
            WorthQueryApplicationObservedFact::AspectPresence {
                entity_id,
                kind,
                aspect,
                present,
            } => {
                let initialized = effects.iter().any(|effect| match effect {
                    WorthQueryApplicationRealizedEffect::UpdateEntity {
                        entity_id: target,
                        fields,
                        ..
                    } if target == entity_id => fields
                        .keys()
                        .any(|locator| locator.aspect().aspect_key() == aspect),
                    WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                        entity_id: target,
                        fields,
                        ..
                    } if target == entity_id => fields.iter().any(|(locator, write)| {
                        locator.aspect().aspect_key() == aspect && write.value.is_some()
                    }),
                    _ => false,
                });
                WorthQueryApplicationObservedFact::AspectPresence {
                    entity_id: *entity_id,
                    kind: *kind,
                    aspect: aspect.clone(),
                    present: *present || initialized,
                }
            }
            WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator,
                value,
            } => match replacement_field_value(*entity_id, locator, effects) {
                Some(Some(replacement)) => WorthQueryApplicationObservedFact::Field {
                    entity_id: *entity_id,
                    kind: *kind,
                    locator: locator.clone(),
                    value: replacement.clone(),
                },
                Some(None) => WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: *entity_id,
                    kind: *kind,
                    locator: locator.clone(),
                },
                None => WorthQueryApplicationObservedFact::Field {
                    entity_id: *entity_id,
                    kind: *kind,
                    locator: locator.clone(),
                    value: value.clone(),
                },
            },
            WorthQueryApplicationObservedFact::AbsentField {
                entity_id,
                kind,
                locator,
            } => match replacement_field_value(*entity_id, locator, effects) {
                Some(Some(replacement)) => WorthQueryApplicationObservedFact::Field {
                    entity_id: *entity_id,
                    kind: *kind,
                    locator: locator.clone(),
                    value: replacement.clone(),
                },
                _ => fact.clone(),
            },
            _ => fact.clone(),
        })
        .collect()
}

#[cfg(test)]
fn replacement_field_value<'effect>(
    entity_id: EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    effects: &'effect [WorthQueryApplicationRealizedEffect],
) -> Option<Option<&'effect worth_foundational::facade::AspectValue>> {
    effects.iter().find_map(|effect| match effect {
        WorthQueryApplicationRealizedEffect::UpdateEntity {
            entity_id: candidate,
            fields,
            ..
        } if *candidate == entity_id => fields.get(locator).map(Some),
        WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
            entity_id: candidate,
            fields,
            ..
        } if *candidate == entity_id => fields.get(locator).map(|write| write.value.as_ref()),
        _ => None,
    })
}

#[cfg(test)]
mod owned_program_facts_tests;
