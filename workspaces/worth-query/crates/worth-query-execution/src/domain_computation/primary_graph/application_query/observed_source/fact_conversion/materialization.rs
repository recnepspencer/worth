//! Convert one validated complete source under its borrowed request controls.
use super::*;

impl<Query> PreparedSourceFactMaterialization<'_, '_, '_, Query> {
    #[expect(
        clippy::mutable_key_type,
        reason = "StructuralWalkKey orders only immutable parameters; its memory lease does not participate in Eq or Ord"
    )]
    pub(in crate::domain_computation::primary_graph::application_query) fn into_facts(
        self,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        let source = self.source;
        let control = self.control;
        control.poll(&source.query_identifier)?;
        let mut facts = reserve_facts(self.fact_count, &source.query_identifier)?;
        let mut fields = self.fields.iter();
        for observed in source.source_meaning.footprint().fact_sources() {
            control.poll(&source.query_identifier)?;
            if let Some(observation) = observed.aggregate {
                facts.push(Fact::Aggregate {
                    observation: std::sync::Arc::clone(observation),
                });
            }
            if let Some(observation) = observed.components {
                facts.push(Fact::ComponentSelection {
                    observation: std::sync::Arc::clone(observation),
                });
            }
            for observation in observed.walks.into_iter().flat_map(|walks| walks.values()) {
                control.poll(&source.query_identifier)?;
                facts.push(Fact::StructuralWalk {
                    observation: std::sync::Arc::clone(observation),
                });
            }
            for entity_id in observed.entities {
                control.poll(&source.query_identifier)?;
                facts.push(Fact::SourceEntity {
                    entity_id: *entity_id,
                });
            }
            for _ in observed.aspects {
                control.poll(&source.query_identifier)?;
                fields
                    .next()
                    .expect("every source field was validated")
                    .append(&mut facts);
            }
            for adjacency in observed.adjacencies {
                control.poll(&source.query_identifier)?;
                facts.push(
                    adjacency_fact(adjacency, control.storage()).map_err(|denial| {
                        WorthQuerySourceExpectationDenial::source_retention_denied(
                            &source.query_identifier,
                            denial,
                        )
                    })?,
                );
            }
        }
        merge_facts(facts, control, &source.query_identifier)
    }
}

#[expect(
    clippy::mutable_key_type,
    reason = "Locator keys order immutable source coordinates"
)]
pub(super) fn merge_facts(
    facts: Vec<Fact>,
    control: SourceFactExpansionControl<'_, '_>,
    subject: &str,
) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
    let mut seen = std::collections::BTreeMap::new();
    let mut unique = reserve_facts(facts.len(), subject)?;
    for fact in facts {
        control.poll(subject)?;
        let identity = SourceFactLocator::from_fact(&fact);
        match seen.entry(identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(unique.len());
                unique.push(fact);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let existing = &mut unique[*entry.get()];
                if !existing
                    .merge_same_source_fact(
                        fact,
                        control.storage().policy(),
                        control.storage().request(),
                    )
                    .map_err(|denial| {
                        WorthQuerySourceExpectationDenial::source_retention_denied(subject, denial)
                    })?
                {
                    return Err(WorthQuerySourceExpectationDenial::new(
                        WorthQuerySourceExpectationDenialKind::SourceChanged,
                        subject,
                    ));
                }
            }
        }
    }
    control.poll(subject)?;
    Ok(unique)
}

fn reserve_facts(
    count: usize,
    subject: &str,
) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
    let mut facts = Vec::new();
    facts.try_reserve_exact(count).map_err(|_| {
        WorthQuerySourceExpectationDenial::new(
            WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
            subject,
        )
    })?;
    Ok(facts)
}
