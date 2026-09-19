use std::collections::{BTreeMap, BTreeSet};

use crate::declaration::UiAspectName;
use crate::fact_contract::UiProducedFact;
use crate::graph::{
    UiGraphFactConsumerIdentity, UiGraphFactConsumerKey, UiGraphFactIndexBasis,
    UiGraphFactIndexEntry, UiGraphFactLookupDenial, UiGraphFactLookupReceipt,
};
use crate::runtime::observation::UiClassifiedChange;
use crate::runtime::rebind::{UiRebindBudgetInput, UiRebindLimit};

use super::{
    UiAffectedConsumer, UiAffectedFactLookup, UiAffectedScopeBasis, UiAffectedScopeCost,
    UiAffectedScopeCostInput, UiAffectedScopeDenial, UiAffectedScopeGeneration,
    UiAffectedScopeRecoveryStop, UiResolvedAffectedScope, UiResolvedAffectedScopeInput,
};

pub(crate) struct UiAffectedScopeResolver;

pub(super) struct ConsumerAccumulator {
    predecessor: Option<UiGraphFactConsumerIdentity>,
    candidate: Option<UiGraphFactConsumerIdentity>,
    aspects: BTreeSet<UiAspectName>,
}

pub(super) struct FinishScopeInput {
    pub(super) classification: crate::runtime::observation::UiChangeClassificationBasis,
    pub(super) facts: Box<[UiProducedFact]>,
    pub(super) source_succession: Option<crate::runtime::observation::UiAuthoredSourceSuccession>,
    pub(super) theme_switch: Option<crate::runtime::appearance::UiThemeSwitchChange>,
    pub(super) predecessor_graph: UiGraphFactIndexBasis,
    pub(super) candidate_generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(super) candidate_graph: UiGraphFactIndexBasis,
    pub(super) lookups: Vec<UiAffectedFactLookup>,
    pub(super) consumers: BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
    pub(super) aspects: BTreeSet<UiAspectName>,
}

struct DualGenerationLookupInput<'world> {
    fact_ordinal: usize,
    fact: &'world UiProducedFact,
    predecessor:
        &'world crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    candidate:
        &'world crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    predecessor_basis: UiGraphFactIndexBasis,
    candidate_basis: UiGraphFactIndexBasis,
}

struct ScopeLookupInput<'world> {
    facts: &'world [UiProducedFact],
    predecessor:
        &'world crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    candidate:
        &'world crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    predecessor_basis: UiGraphFactIndexBasis,
    candidate_basis: UiGraphFactIndexBasis,
    budget: UiRebindBudgetInput,
}

pub(super) struct ScopeAccumulation {
    pub(super) lookups: Vec<UiAffectedFactLookup>,
    pub(super) consumers: BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
    pub(super) aspects: BTreeSet<UiAspectName>,
}

impl UiAffectedScopeResolver {
    pub(crate) fn resolve(
        change: UiClassifiedChange,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        predecessor: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationAuthority,
    ) -> Result<UiResolvedAffectedScope, UiAffectedScopeDenial> {
        Self::resolve_recoverable(change, session, predecessor)
            .map_err(UiAffectedScopeRecoveryStop::into_denial)
    }
}

pub(super) struct PreparedScopeResolution {
    pub(super) predecessor_graph: UiGraphFactIndexBasis,
    pub(super) candidate_generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(super) candidate_graph: UiGraphFactIndexBasis,
    pub(super) accumulation: ScopeAccumulation,
}

pub(super) fn prepare_resolution(
    change: &UiClassifiedChange,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    predecessor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
) -> Result<PreparedScopeResolution, UiAffectedScopeDenial> {
    require_current_basis(change.basis(), session, predecessor)?;
    let candidate = change
        .source_succession()
        .map_or(predecessor, |succession| succession.successor_authority());
    let predecessor_graph = UiGraphFactIndexBasis::from_generation(
        predecessor.graph_snapshot(),
        predecessor.capabilities(),
    );
    let candidate_graph = UiGraphFactIndexBasis::from_generation(
        candidate.graph_snapshot(),
        candidate.capabilities(),
    );
    let candidate_generation = candidate.generation_identity().clone();
    let budget = predecessor.change_profile().rebind().budget();
    enforce_limit(
        UiRebindLimit::ChangedFacts,
        budget.changed_facts,
        change.facts().len(),
    )?;

    let mut accumulation = accumulate_scope(ScopeLookupInput {
        facts: change.facts(),
        predecessor,
        candidate,
        predecessor_basis: predecessor_graph,
        candidate_basis: candidate_graph,
        budget,
    })?;

    if let Some(theme) = change.theme_switch() {
        for (node, _) in theme.invalidation().mounted_consumers() {
            let snapshot = predecessor.graph_snapshot();
            let node = snapshot
                .core_indexes()
                .node_identity()
                .node(snapshot.nodes(), *node)
                .expect("theme selection carries a current indexed consumer");
            let identity = UiGraphFactConsumerIdentity::GraphNode(node.graph_node_identity());
            let key = UiGraphFactConsumerKey::new(
                crate::graph::UiGraphFactConsumerKind::GraphNode,
                node.declaration_identity().authored_semantic_name(),
                node.repeated_instance_basis().identity_digest(),
            );
            accumulation
                .consumers
                .entry(key)
                .or_insert_with(|| ConsumerAccumulator {
                    predecessor: Some(identity),
                    candidate: Some(identity),
                    aspects: BTreeSet::new(),
                });
        }
        enforce_scope_limits(&accumulation.consumers, &accumulation.aspects, budget)?;
        enforce_limit(
            UiRebindLimit::GraphAndMountedEntries,
            budget.graph_and_mounted_entries,
            selected_entry_count(&accumulation.consumers) + theme.cost().graph_and_mounted_entries,
        )?;
    }
    Ok(PreparedScopeResolution {
        predecessor_graph,
        candidate_generation,
        candidate_graph,
        accumulation,
    })
}

fn accumulate_scope(
    input: ScopeLookupInput<'_>,
) -> Result<ScopeAccumulation, UiAffectedScopeDenial> {
    let mut accumulation = ScopeAccumulation {
        lookups: Vec::with_capacity(input.facts.len()),
        consumers: BTreeMap::new(),
        aspects: BTreeSet::new(),
    };
    for (fact_ordinal, fact) in input.facts.iter().enumerate() {
        let (predecessor, candidate) = lookup_both_generations(DualGenerationLookupInput {
            fact_ordinal,
            fact,
            predecessor: input.predecessor,
            candidate: input.candidate,
            predecessor_basis: input.predecessor_basis,
            candidate_basis: input.candidate_basis,
        })?;
        join_entries(
            predecessor.entries(),
            UiAffectedScopeGeneration::Predecessor,
            &mut accumulation.consumers,
            &mut accumulation.aspects,
        );
        join_entries(
            candidate.entries(),
            UiAffectedScopeGeneration::Candidate,
            &mut accumulation.consumers,
            &mut accumulation.aspects,
        );
        enforce_scope_limits(&accumulation.consumers, &accumulation.aspects, input.budget)?;
        accumulation.lookups.push(UiAffectedFactLookup::new(
            fact_ordinal,
            fact.family(),
            predecessor,
            candidate,
        ));
    }
    Ok(accumulation)
}

fn require_current_basis(
    basis: &crate::runtime::observation::UiChangeClassificationBasis,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    predecessor: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationAuthority,
) -> Result<(), UiAffectedScopeDenial> {
    if basis.session() != session {
        return Err(UiAffectedScopeDenial::ForeignSession);
    }
    if basis.source_basis() != predecessor.capabilities().digest().as_u64() {
        return Err(UiAffectedScopeDenial::StaleSourceBasis);
    }
    if basis.predecessor_generation() != predecessor.generation_identity() {
        return Err(UiAffectedScopeDenial::StalePredecessorGeneration);
    }
    Ok(())
}

fn lookup_both_generations(
    input: DualGenerationLookupInput<'_>,
) -> Result<(UiGraphFactLookupReceipt, UiGraphFactLookupReceipt), UiAffectedScopeDenial> {
    let DualGenerationLookupInput {
        fact_ordinal,
        fact,
        predecessor,
        candidate,
        predecessor_basis,
        candidate_basis,
    } = input;
    let predecessor_lookup = predecessor
        .consumed_fact_index()
        .lookup(predecessor_basis, fact);
    let candidate_lookup = candidate
        .consumed_fact_index()
        .lookup(candidate_basis, fact);
    match (predecessor_lookup, candidate_lookup) {
        (Ok(predecessor), Ok(candidate)) => Ok((predecessor, candidate)),
        (Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration { .. }), Ok(candidate))
            if predecessor_basis != candidate_basis =>
        {
            Ok((
                UiGraphFactLookupReceipt::new(predecessor_basis, Box::new([])),
                candidate,
            ))
        }
        (Ok(predecessor), Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration { .. }))
            if predecessor_basis != candidate_basis =>
        {
            Ok((
                predecessor,
                UiGraphFactLookupReceipt::new(candidate_basis, Box::new([])),
            ))
        }
        (
            Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration { authored_identity }),
            Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration { .. }),
        ) => Err(
            UiAffectedScopeDenial::UnknownAuthoredSelectorInBothGenerations {
                fact_ordinal,
                authored_identity,
            },
        ),
        (Err(source), _) => Err(UiAffectedScopeDenial::Index {
            generation: UiAffectedScopeGeneration::Predecessor,
            fact_ordinal,
            source,
        }),
        (_, Err(source)) => Err(UiAffectedScopeDenial::Index {
            generation: UiAffectedScopeGeneration::Candidate,
            fact_ordinal,
            source,
        }),
    }
}

fn join_entries(
    entries: &[UiGraphFactIndexEntry],
    generation: UiAffectedScopeGeneration,
    consumers: &mut BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
    aspects: &mut BTreeSet<UiAspectName>,
) {
    for entry in entries {
        let consumer = consumers
            .entry(entry.consumer_key().clone())
            .or_insert_with(|| ConsumerAccumulator {
                predecessor: None,
                candidate: None,
                aspects: BTreeSet::new(),
            });
        match generation {
            UiAffectedScopeGeneration::Predecessor => consumer.predecessor = Some(entry.consumer()),
            UiAffectedScopeGeneration::Candidate => consumer.candidate = Some(entry.consumer()),
        }
        if let Some(aspect) = entry.affected_aspect() {
            consumer.aspects.insert(aspect.clone());
            aspects.insert(aspect.clone());
        }
    }
}

fn enforce_scope_limits(
    consumers: &BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
    aspects: &BTreeSet<UiAspectName>,
    budget: UiRebindBudgetInput,
) -> Result<(), UiAffectedScopeDenial> {
    enforce_limit(
        UiRebindLimit::AffectedAspects,
        budget.affected_aspects,
        aspects.len(),
    )?;
    enforce_limit(
        UiRebindLimit::DistinctConsumers,
        budget.distinct_consumers,
        consumers.len(),
    )?;
    enforce_limit(
        UiRebindLimit::GraphAndMountedEntries,
        budget.graph_and_mounted_entries,
        selected_entry_count(consumers),
    )
}

fn enforce_limit(
    limit: UiRebindLimit,
    configured: usize,
    observed: usize,
) -> Result<(), UiAffectedScopeDenial> {
    if observed > configured {
        Err(UiAffectedScopeDenial::BudgetExceeded {
            limit,
            configured,
            observed,
        })
    } else {
        Ok(())
    }
}

fn selected_entry_count(
    consumers: &BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
) -> usize {
    consumers
        .values()
        .map(|consumer| {
            usize::from(consumer.predecessor.is_some()) + usize::from(consumer.candidate.is_some())
        })
        .sum()
}

mod materialization;
pub(super) use materialization::finish_scope;
