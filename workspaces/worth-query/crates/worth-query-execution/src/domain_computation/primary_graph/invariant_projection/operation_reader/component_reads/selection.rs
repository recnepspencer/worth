//! Whole-cohort visibility precedes all typed membership and identity output.
use super::*;
use crate::domain_computation::primary_graph::root_path_selection::RootPathOrientation;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Of visits only the selected cohort and reverse witnesses. Census visits
    /// the complete forward union, including negative and empty-set evidence.
    pub(super) fn collect_component_selection<Entity>(
        &mut self,
        scope: &WorthQueryOperationComponentReadScope,
        layout: &ComponentSetLayout,
        root: EntityId,
        member: Option<EntityId>,
        pin: &RelationalPinnedRead,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        subject: &str,
    ) -> Result<(Vec<Arc<AggregateObservation>>, Vec<SourceFacts>), Denial> {
        let mut facts = Vec::new();
        let universe = if let Some(member) = member {
            BTreeSet::from([member])
        } else {
            let selected = self.select_component_paths(
                scope,
                root,
                RootPathOrientation::ForwardCensus,
                true,
                pin,
                subject,
            )?;
            let universe = selected.roots.into_keys().collect();
            facts.push(
                selected
                    .set_source
                    .expect("census captures the complete union")
                    .finish()
                    .map_err(|cause| self.reader.walk_stop(cause, subject))?,
            );
            universe
        };
        let mut seen = BTreeSet::new();
        let mut aggregates = Vec::new();
        for anchor in &universe {
            AggregateObservation::checkpoint(context)
                .map_err(|cause| self.reader.walk_stop(cause, subject))?;
            if seen.contains(anchor) {
                continue;
            }
            let aggregate = self.reader.read_aggregate_identity(
                layout.member_kind,
                AggregateSelection::ComponentDefinition(layout.definition),
                *anchor,
                subject,
            )?;
            for candidate in aggregate.native().members() {
                AggregateObservation::checkpoint(context)
                    .map_err(|cause| self.reader.walk_stop(cause, subject))?;
                if member.is_some() {
                    let mut selected = self.select_component_paths(
                        scope,
                        candidate,
                        RootPathOrientation::ReverseWitness {
                            required_root: root,
                        },
                        false,
                        pin,
                        subject,
                    )?;
                    let witness = selected
                        .roots
                        .remove(&root)
                        .ok_or_else(|| Denial::new(Kind::ComponentOutsideScope, subject))?;
                    facts.push(
                        witness
                            .finish()
                            .map_err(|cause| self.reader.walk_stop(cause, subject))?,
                    );
                } else if !universe.contains(&candidate) {
                    return Err(Denial::new(Kind::ComponentOutsideScope, subject));
                }
                seen.insert(candidate);
            }
            self.reader
                .admit_component_members::<Entity>(layout, &aggregate, context, subject)?;
            facts.push(self.component_coupling_facts(layout, &aggregate, pin, context, subject)?);
            aggregates.push(aggregate);
        }
        aggregates.sort_by_key(|body| body.native().identity());
        Ok((aggregates, facts))
    }
    fn component_coupling_facts(
        &mut self,
        layout: &ComponentSetLayout,
        aggregate: &AggregateObservation,
        pin: &RelationalPinnedRead,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        subject: &str,
    ) -> Result<SourceFacts, Denial> {
        let projection = pin
            .project_snapshot(self.reader.snapshot)
            .ok_or_else(|| Denial::new(Kind::EndpointUnavailable, subject))?;
        let mut facts = SourceFactsBuilder::default();
        for member in aggregate.native().members() {
            AggregateObservation::checkpoint(context)
                .map_err(|cause| self.reader.walk_stop(cause, subject))?;
            facts.observe_entity(member);
            for (_, kind) in layout.sources.iter() {
                for direction in [
                    RelationalAdjacencyDirection::Outgoing,
                    RelationalAdjacencyDirection::Incoming,
                ] {
                    facts
                        .observe_adjacency(
                            self.reader.execution,
                            &projection,
                            member,
                            *kind,
                            direction,
                        )
                        .map_err(|cause| self.component_source_denial(cause, subject))?;
                }
            }
        }
        facts
            .finish()
            .map_err(|cause| self.reader.walk_stop(cause, subject))
    }
}
