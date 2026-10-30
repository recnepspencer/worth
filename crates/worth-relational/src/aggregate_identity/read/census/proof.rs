//! Whole-census custody, including an empty universe, from one genuine view.
use super::*;
use crate::derived_relation::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle};
use crate::projected_member_domain::ProjectedMemberDomainHandle;
use crate::schema::SchemaRegistryCustody;
use crate::validation::engine::InvariantSourceAffinity;
use std::hash::{Hash, Hasher};

/// Opaque exact-view lookup currency; descriptive definition bytes cannot issue it.
#[derive(Clone)]
pub struct AggregateCensusReadIdentity {
    source: InvariantSourceAffinity,
    registry: SchemaRegistryCustody,
    definition: AggregateDefinitionIdentity,
    derived: Option<DerivedRelationHandle>,
    domain: Option<ProjectedMemberDomainHandle>,
    projected: bool,
}
impl PartialEq for AggregateCensusReadIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && std::ptr::eq(self.registry.registry(), other.registry.registry())
            && self.definition == other.definition
            && self.derived == other.derived
            && self.domain == other.domain
            && self.projected == other.projected
    }
}
impl Eq for AggregateCensusReadIdentity {}
impl Hash for AggregateCensusReadIdentity {
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.observation.hash(state);
        self.source.committed.hash(state);
        self.source.version.hash(state);
        std::ptr::from_ref(self.registry.registry()).hash(state);
        self.definition.hash(state);
        self.projected.hash(state);
        // Full Eq also checks proposal and actual registered source/domain.
    }
}

/// Retains the actual census and original payer; no rows can be substituted.
#[derive(Clone)]
pub struct PreparedAggregateCensus<Member = EntityId> {
    identity: AggregateCensusReadIdentity,
    census: Arc<AggregateReadCollection<Member>>,
}
/// Read-only census returned only after the current view consumes its proof.
#[derive(Clone)]
pub struct AggregateCensusRead<Member = EntityId> {
    census: Arc<AggregateReadCollection<Member>>,
}
impl<Member> AggregateCensusRead<Member> {
    pub fn collection(&self) -> &AggregateReadCollection<Member> {
        &self.census
    }
    pub fn len(&self) -> usize {
        self.census.len()
    }
    pub fn is_empty(&self) -> bool {
        self.census.is_empty()
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &AggregateRead<Member>> {
        self.census.iter()
    }
}
impl<Member> PreparedAggregateCensus<Member> {
    pub fn identity(&self) -> &AggregateCensusReadIdentity {
        &self.identity
    }
}

impl AggregateReadPort<'_, '_> {
    /// Validate the complete contract/source audience without enumerating records.
    pub fn census_read_identity(
        &self,
        definition: AggregateDefinitionIdentity,
        projected: bool,
    ) -> Result<AggregateCensusReadIdentity, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        let (_, _, aggregate) = self.view.aggregate_read_parts();
        let (aggregate, committed) = aggregate.ok_or(AggregateReadDenial::SourceUnavailable)?;
        let (registry, source) = self
            .view
            .structural_source
            .as_ref()
            .ok_or(AggregateReadDenial::SourceUnavailable)?;
        if !std::ptr::eq(registry.registry(), aggregate.registry(committed)) {
            return Err(AggregateReadDenial::Derived(
                DefinitionDenial::ForeignDefinition.into(),
            ));
        }
        budget
            .checkpoint(aggregate.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = aggregate
            .contract(AggregateSelection::Definition(definition), committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        self.view
            .require_entity_kind(contract.member_kind)
            .map_err(|cause| AggregateReadDenial::Budget(budget.refusal(cause)))?;
        let derived = self.require_source(contract, aggregate.registry(committed))?;
        let domain = match derived.as_ref().map(|source| &source.parts().from) {
            Some(crate::graph_member::NativeMemberDomain::Projected {
                projected: identity,
            }) => {
                if !projected {
                    return Err(AggregateReadDenial::WrongMemberKind);
                }
                // require_source admitted all exact registered carriers and
                // charged this recipe/domain name lookup through its owner.
                let names = identity
                    .name_bytes()
                    .ok_or(AggregateReadDenial::PreparationRefused)?;
                let levels = usize::BITS
                    - registry
                        .registry()
                        .projected_member_domains
                        .len()
                        .max(1)
                        .leading_zeros();
                let work = (names as u64)
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(u64::from(levels)))
                    .ok_or(AggregateReadDenial::PreparationRefused)?;
                budget
                    .checkpoint(work)
                    .map_err(AggregateReadDenial::Budget)?;
                Some(
                    registry
                        .registry()
                        .projected_member_domain(identity)
                        .ok_or(AggregateReadDenial::WrongMemberKind)?,
                )
            }
            _ if projected => return Err(AggregateReadDenial::WrongMemberKind),
            _ => None,
        };
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        Ok(AggregateCensusReadIdentity {
            source: source.clone(),
            registry: registry.clone(),
            definition,
            derived,
            domain,
            projected,
        })
    }

    pub fn prepare_census_of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
    ) -> Result<PreparedAggregateCensus, AggregateReadDenial> {
        let identity = self.census_read_identity(definition, false)?;
        let census = self.all_of_definition(definition)?;
        self.seal_census(identity, census)
    }
    pub fn prepare_projected_census_of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
    ) -> Result<PreparedAggregateCensus<NativeMemberKey>, AggregateReadDenial> {
        let identity = self.census_read_identity(definition, true)?;
        let census = self.all_projected_of_definition(definition)?;
        self.seal_census(identity, census)
    }
    fn seal_census<Member>(
        &self,
        identity: AggregateCensusReadIdentity,
        census: AggregateReadCollection<Member>,
    ) -> Result<PreparedAggregateCensus<Member>, AggregateReadDenial> {
        self.view
            .read_budget()
            .claim_retained(
                (size_of::<AggregateReadCollection<Member>>() + 2 * size_of::<usize>()) as u64,
            )
            .map_err(AggregateReadDenial::Budget)?;
        self.view
            .read_budget()
            .check_live()
            .map_err(AggregateReadDenial::Budget)?;
        Ok(PreparedAggregateCensus {
            identity,
            census: Arc::new(census),
        })
    }
    pub fn read_prepared_census<Member>(
        &self,
        prepared: &PreparedAggregateCensus<Member>,
    ) -> Result<AggregateCensusRead<Member>, AggregateReadDenial> {
        let actual =
            self.census_read_identity(prepared.identity.definition, prepared.identity.projected)?;
        if actual != prepared.identity {
            return Err(AggregateReadDenial::Derived(
                DefinitionDenial::ForeignObservation.into(),
            ));
        }
        Ok(AggregateCensusRead {
            census: Arc::clone(&prepared.census),
        })
    }
}
