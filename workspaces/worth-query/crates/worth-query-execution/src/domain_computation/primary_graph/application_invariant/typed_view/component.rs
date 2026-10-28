//! Complete component reads use Native identity and actual business footprints.
use super::super::{
    WorthQueryApplicationInvariantComponentSetBinding, WorthQueryInvariantAccessDenial,
    WorthQueryInvariantAccessDenialKind,
};
use super::{
    denial, map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantReadView,
};
use worth_relational::facade::runtime::{
    AggregateRead, AggregateReadDenial, StructuralReadError, StructuralWalkDenialKind,
};
pub(super) mod result;
pub use result::{
    WorthQueryApplicationInvariantComponent, WorthQueryApplicationInvariantComponents,
};

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// Select the complete genuine Native cohort of one member. This never
    /// enumerates unrelated components or infers identity from a routing key.
    pub fn component_of<Set, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantComponentSetBinding<Schema, Set, Entity>,
        member: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<
        WorthQueryApplicationInvariantComponent<Schema, Set, Entity>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        if binding.member_domain.entity_kind().is_none() {
            return Err(invalid("entity-only component read"));
        }
        self.check_entity_affinity(member, binding.member_kind)?;
        let prepared = binding
            .source
            .as_ref()
            .map(|source| {
                self.prepared_derived_source(
                    source,
                    member.entity_id.into(),
                    worth_relational::facade::runtime::DerivedRelationReadShape::Component,
                )
            })
            .transpose()?;
        let native = self
            .relations
            .aggregates()
            .of_definition(binding.definition, member.entity_id)
            .map_err(aggregate_denial)?;
        if let Some(prepared) = &prepared {
            self.check_derived_cohort(native.members().map(Into::into), prepared)?;
        } else {
            self.admit_component(binding, &native)?;
        }
        Ok(WorthQueryApplicationInvariantComponent::new(
            native,
            result::ComponentContext::new(self, binding.member_kind),
        ))
    }

    /// Complete census, including isolated members and an honestly empty set.
    /// Deliberate census work may grow with the exact-view Native inventory.
    pub fn components<Set, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantComponentSetBinding<Schema, Set, Entity>,
    ) -> Result<
        WorthQueryApplicationInvariantComponents<Schema, Set, Entity>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        if binding.member_domain.entity_kind().is_none() {
            return Err(invalid("entity-only component census"));
        }
        if binding.source.is_some() {
            return self.derived_entity_components(binding);
        }
        let native = self
            .relations
            .aggregates()
            .all_of_definition(binding.definition)
            .map_err(aggregate_denial)?;
        for group in native.iter() {
            self.admit_component(binding, group)?;
        }
        Ok(WorthQueryApplicationInvariantComponents::new(
            native,
            result::ComponentContext::new(self, binding.member_kind),
        ))
    }
    /// Counting retains the same complete census and physical read footprint.
    pub fn component_count<Set, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantComponentSetBinding<Schema, Set, Entity>,
    ) -> Result<usize, WorthQueryInvariantAccessDenial> {
        Ok(self.components(binding)?.len())
    }

    fn admit_component<Set, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantComponentSetBinding<Schema, Set, Entity>,
        native: &AggregateRead,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        if native.definition() != binding.definition {
            return Err(invalid("component definition affinity"));
        }
        for member in native.members() {
            self.relations
                .read_budget()
                .checkpoint(1)
                .map_err(|cause| {
                    WorthQueryInvariantAccessDenial::from_native_read(cause, "component footprint")
                })?;
            self.check_entity(
                &self.entity::<Entity>(member, binding.member_kind),
                binding.member_kind,
            )?;
            let ids = self
                .relations
                .outgoing_relations_for_entity(member)
                .map_err(|_| {
                    // Ask the original meter for its actual stop rather than erase
                    // cancellation, deadline or memory refusal behind adjacency's error.
                    let cause = self.relations.read_budget().checkpoint(0).err().unwrap_or(
                        StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded),
                    );
                    WorthQueryInvariantAccessDenial::from_native_read(cause, "component adjacency")
                })?;
            self.reads.adjacency();
            for id in ids {
                self.relations
                    .read_budget()
                    .checkpoint(binding.sources.len() as u64)
                    .map_err(|cause| {
                        WorthQueryInvariantAccessDenial::from_native_read(
                            cause,
                            "component coupling",
                        )
                    })?;
                let edge = self
                    .relations
                    .relation(id)
                    .map_err(|cause| map_structural_error(cause, "component physical edge"))?;
                self.reads.edge();
                if !binding
                    .sources
                    .iter()
                    .any(|(_, kind)| *kind == edge.kind_id)
                {
                    continue;
                }
                if edge.source != member || !native.contains(edge.target) {
                    return Err(invalid("component coupling leaves complete Native cohort"));
                }
                self.check_entity(
                    &self.entity::<Entity>(edge.target, binding.member_kind),
                    binding.member_kind,
                )?;
                self.admission.relation(id, self.relations.read_budget())?;
            }
        }
        self.relations.read_budget().check_live().map_err(|cause| {
            WorthQueryInvariantAccessDenial::from_native_read(cause, "component footprint")
        })
    }
}
fn invalid(subject: &str) -> WorthQueryInvariantAccessDenial {
    denial(
        WorthQueryInvariantAccessDenialKind::InvalidComponent,
        subject,
    )
}
pub(in crate::domain_computation::primary_graph::application_invariant) fn aggregate_denial(
    cause: AggregateReadDenial,
) -> WorthQueryInvariantAccessDenial {
    use WorthQueryInvariantAccessDenialKind as Kind;
    let kind = match cause {
        AggregateReadDenial::Budget(cause) => {
            return WorthQueryInvariantAccessDenial::from_native_read(cause, "component read")
        }
        AggregateReadDenial::SourceUnavailable => Kind::AggregateSourceUnavailable,
        AggregateReadDenial::NotAggregateRelation => Kind::RelationNotInstalled,
        AggregateReadDenial::WrongMemberKind => Kind::WrongEntityKind,
        AggregateReadDenial::PreparationRefused => Kind::AggregatePreparationRefused,
        AggregateReadDenial::Execution(cause) => Kind::Execution(cause),
        AggregateReadDenial::Derived(cause) => {
            return super::derived_relation::native_denial(cause)
        }
    };
    denial(kind, "component read")
}
