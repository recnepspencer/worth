use super::*;
pub(in crate::runtime::rebind::scope) fn finish_scope(
    input: FinishScopeInput,
) -> Result<UiResolvedAffectedScope, UiAffectedScopeDenial> {
    let FinishScopeInput {
        classification,
        facts,
        source_succession,
        theme_switch,
        predecessor_graph,
        candidate_generation,
        candidate_graph,
        lookups,
        consumers,
        aspects,
    } = input;
    let indexed_consumers = consumers.len();
    let theme_cost = theme_switch
        .as_ref()
        .map(|theme| theme.cost())
        .unwrap_or_default();
    let graph_and_mounted_entries =
        selected_entry_count(&consumers) + theme_cost.graph_and_mounted_entries;
    let (index_probes, contract_checks) = lookup_cost(&lookups);
    let index_probes = index_probes + theme_cost.index_probes;
    let affected_aspects = aspects.into_iter().collect::<Vec<_>>().into_boxed_slice();
    let consumers = materialize_consumers(consumers);
    let basis = UiAffectedScopeBasis::new(
        classification,
        predecessor_graph,
        candidate_generation,
        candidate_graph,
    );
    let cost = UiAffectedScopeCost::exact(UiAffectedScopeCostInput {
        observations: basis.classification().observation_count(),
        changed_facts: facts.len(),
        affected_aspects: affected_aspects.len(),
        indexed_consumers,
        lookup_receipts: lookups.len() * 2,
        index_probes,
        contract_checks,
        graph_and_mounted_entries,
        theme_slots_compared: theme_cost.theme_slots_compared,
    });
    Ok(UiResolvedAffectedScope::new(UiResolvedAffectedScopeInput {
        basis,
        facts,
        affected_aspects,
        consumers,
        lookups: lookups.into_boxed_slice(),
        cost,
        source_succession,
        theme_switch,
    }))
}

fn materialize_consumers(
    consumers: BTreeMap<UiGraphFactConsumerKey, ConsumerAccumulator>,
) -> Box<[UiAffectedConsumer]> {
    consumers
        .into_iter()
        .map(|(key, consumer)| {
            UiAffectedConsumer::new(
                key,
                consumer.predecessor,
                consumer.candidate,
                consumer
                    .aspects
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn lookup_cost(lookups: &[UiAffectedFactLookup]) -> (usize, usize) {
    lookups.iter().fold((0, 0), |cost, lookup| {
        (
            cost.0
                + lookup.predecessor().cost().index_probes()
                + lookup.candidate().cost().index_probes(),
            cost.1
                + lookup.predecessor().cost().contract_checks()
                + lookup.candidate().cost().contract_checks(),
        )
    })
}
