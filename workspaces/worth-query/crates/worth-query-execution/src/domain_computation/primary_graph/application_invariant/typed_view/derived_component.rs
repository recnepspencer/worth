//! Derived cohorts retain actual Native identity and complete prepared support.
use super::super::{
    prepared_scope::PreparedComponentCensus as Census,
    WorthQueryApplicationInvariantComponentSetBinding as Binding,
    WorthQueryInvariantAccessDenial as Denial, WorthQueryInvariantAccessDenialKind as Kind,
};
use super::component::{
    aggregate_denial,
    result::{ComponentContext, WorthQueryApplicationInvariantComponents},
};
use super::{
    WorthQueryApplicationInvariantGraphMember as Member,
    WorthQueryApplicationInvariantReadView as View,
};
use crate::domain_computation::primary_graph::{
    aggregate_identity::WorthQueryAggregateIdentity as Identity,
    schema_layout::MemberDomainBinding as Domain,
};
use worth_relational::facade::{
    runtime::{
        AggregateRead, DerivedRelationReadShape as Shape, PreparedDerivedRelationRead as Prepared,
    },
    schema::NativeMemberKey as Key,
};

enum Cohort {
    Entity(AggregateRead),
    Projected(AggregateRead<Key>),
}
/// Complete read-only logical cohort. Identity and membership come from Native.
pub struct WorthQueryApplicationInvariantGraphComponent<S, Set, M> {
    members: Vec<Member<S, M>>,
    identity: Identity<S, Set>,
    _native: Cohort,
    _source: Prepared,
}
impl<S, Set, M> WorthQueryApplicationInvariantGraphComponent<S, Set, M> {
    pub fn identity(&self) -> &Identity<S, Set> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn members(&self) -> &[Member<S, M>] {
        &self.members
    }
}
/// Complete exact-view census, including its genuine empty-universe proof.
pub struct WorthQueryApplicationInvariantGraphComponents<S, Set, M> {
    groups: Vec<WorthQueryApplicationInvariantGraphComponent<S, Set, M>>,
    _census: Census,
}
impl<S, Set, M> WorthQueryApplicationInvariantGraphComponents<S, Set, M> {
    pub fn len(&self) -> usize {
        self.groups.len()
    }
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryApplicationInvariantGraphComponent<S, Set, M>>
    {
        self.groups.iter()
    }
}

impl<S> View<'_, S> {
    /// Select one actual derived cohort from a genuine entity or logical member.
    pub fn graph_component_of<Set, M>(
        &self,
        binding: &Binding<S, Set, M>,
        member: &Member<S, M>,
    ) -> Result<WorthQueryApplicationInvariantGraphComponent<S, Set, M>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        let root = self.check_member(member, &binding.member_domain)?;
        let source = binding
            .source
            .as_ref()
            .ok_or_else(|| invalid("derived component source"))?;
        // Frozen source membership is checked before aggregate acquisition.
        let prepared = self.prepared_derived_source(source, root, Shape::Component)?;
        let port = self.relations.aggregates();
        let native = match &binding.member_domain {
            Domain::Entity(_) => Cohort::Entity(
                port.of_definition(binding.definition, member.carrier_id())
                    .map_err(aggregate_denial)?,
            ),
            Domain::Projected(_) => Cohort::Projected(
                port.of_projected_definition(
                    binding.definition,
                    member
                        .projected_observation()
                        .ok_or_else(|| invalid("projected component member"))?,
                )
                .map_err(aggregate_denial)?,
            ),
        };
        self.graph_component(binding, native, prepared)
    }

    /// Complete census consumes the Native-issued proof before any evaluation acquisition.
    pub fn graph_components<Set, M>(
        &self,
        binding: &Binding<S, Set, M>,
    ) -> Result<WorthQueryApplicationInvariantGraphComponents<S, Set, M>, Denial> {
        self.check_binding(&binding.binding_identity)?;
        let source = binding
            .source
            .as_ref()
            .ok_or_else(|| invalid("derived component source"))?;
        let census = self.prepared_component_census(binding)?;
        let mut groups = Vec::new();
        let port = self.relations.aggregates();
        match &census {
            Census::Entity(proof) => {
                let native = port.read_prepared_census(proof).map_err(aggregate_denial)?;
                self.retain_derived_result::<WorthQueryApplicationInvariantGraphComponent<S, Set, M>>(native.len())?;
                groups.reserve_exact(native.len());
                for group in native.iter() {
                    self.derived_result_checkpoint()?;
                    let prepared = self.prepared_derived_source(
                        source,
                        group.anchor().into(),
                        Shape::Component,
                    )?;
                    groups.push(self.graph_component(
                        binding,
                        Cohort::Entity(group.clone()),
                        prepared,
                    )?);
                }
            }
            Census::Projected(proof) => {
                let native = port.read_prepared_census(proof).map_err(aggregate_denial)?;
                self.retain_derived_result::<WorthQueryApplicationInvariantGraphComponent<S, Set, M>>(native.len())?;
                groups.reserve_exact(native.len());
                for group in native.iter() {
                    self.derived_result_checkpoint()?;
                    let prepared = self.prepared_derived_source(
                        source,
                        group.anchor().clone(),
                        Shape::Component,
                    )?;
                    groups.push(self.graph_component(
                        binding,
                        Cohort::Projected(group.clone()),
                        prepared,
                    )?);
                }
            }
        }
        self.derived_result_checkpoint()?;
        Ok(WorthQueryApplicationInvariantGraphComponents {
            groups,
            _census: census,
        })
    }

    pub(super) fn derived_entity_components<Set, E>(
        &self,
        binding: &Binding<S, Set, E>,
    ) -> Result<WorthQueryApplicationInvariantComponents<S, Set, E>, Denial> {
        let source = binding
            .source
            .as_ref()
            .ok_or_else(|| invalid("derived component source"))?;
        let Census::Entity(proof) = self.prepared_component_census(binding)? else {
            return Err(invalid("entity-only component census"));
        };
        let native = self
            .relations
            .aggregates()
            .read_prepared_census(&proof)
            .map_err(aggregate_denial)?;
        for group in native.iter() {
            let prepared =
                self.prepared_derived_source(source, group.anchor().into(), Shape::Component)?;
            self.check_derived_cohort(group.members().map(Into::into), &prepared)?;
        }
        Ok(WorthQueryApplicationInvariantComponents::from_prepared(
            native,
            ComponentContext::new(self, binding.member_kind),
        ))
    }

    fn prepared_component_census<Set, M>(
        &self,
        binding: &Binding<S, Set, M>,
    ) -> Result<Census, Denial> {
        let projected = matches!(binding.member_domain, Domain::Projected(_));
        let key = self
            .relations
            .aggregates()
            .census_read_identity(binding.definition, projected)
            .map_err(aggregate_denial)?;
        self.admission
            .component_census(key, &self.relations, binding.definition, projected)
    }

    fn graph_component<Set, M>(
        &self,
        binding: &Binding<S, Set, M>,
        native: Cohort,
        prepared: Prepared,
    ) -> Result<WorthQueryApplicationInvariantGraphComponent<S, Set, M>, Denial> {
        let (identity, members) = match &native {
            Cohort::Entity(group) => {
                if group.definition() != binding.definition {
                    return Err(invalid("component definition"));
                }
                self.check_derived_cohort(group.members().map(Into::into), &prepared)?;
                (
                    Identity::from_invariant_component_read(group, self.binding_identity.clone()),
                    self.issue_component_members(
                        group.members().map(Into::into),
                        &binding.member_domain,
                    )?,
                )
            }
            Cohort::Projected(group) => {
                if group.definition() != binding.definition {
                    return Err(invalid("component definition"));
                }
                self.check_derived_cohort(group.members().cloned(), &prepared)?;
                (
                    Identity::from_invariant_component_read(group, self.binding_identity.clone()),
                    self.issue_component_members(group.members().cloned(), &binding.member_domain)?,
                )
            }
        };
        Ok(WorthQueryApplicationInvariantGraphComponent {
            identity,
            members,
            _native: native,
            _source: prepared,
        })
    }

    fn issue_component_members<M>(
        &self,
        members: impl ExactSizeIterator<Item = Key>,
        domain: &Domain,
    ) -> Result<Vec<Member<S, M>>, Denial> {
        self.retain_derived_result::<Member<S, M>>(members.len())?;
        let mut result = Vec::with_capacity(members.len());
        for key in members {
            self.derived_result_checkpoint()?;
            result.push(self.issue_member(&key, domain)?);
        }
        self.derived_result_checkpoint()?;
        Ok(result)
    }

    pub(super) fn check_derived_cohort(
        &self,
        members: impl ExactSizeIterator<Item = Key>,
        prepared: &Prepared,
    ) -> Result<(), Denial> {
        let data = self
            .relations
            .read_prepared_derived_source(prepared, Shape::Component)
            .map_err(super::derived_relation::native_denial)?;
        if members.len() != data.endpoints().len() {
            return Err(invalid("complete component cardinality"));
        }
        for (actual, expected) in members.zip(data.endpoints()) {
            let names = match &actual {
                Key::Entity(_) => Some(0),
                Key::Projected(member) => member
                    .domain
                    .schema
                    .0
                    .len()
                    .checked_add(member.domain.name.len())
                    .and_then(|n| n.checked_add(member.case.len())),
            }
            .ok_or_else(|| invalid("component member size"))?;
            self.relations
                .read_budget()
                .checkpoint((names as u64).saturating_add(1))
                .map_err(|cause| Denial::from_native_read(cause, "component source equality"))?;
            if &actual != expected {
                return Err(invalid("complete component source equality"));
            }
        }
        self.derived_result_checkpoint()
    }
}
fn invalid(subject: &str) -> Denial {
    Denial::new(Kind::InvalidComponent, subject)
}
