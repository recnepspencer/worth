//! Coupling admission acquires every physical direction through ordinary readers.
use super::*;
use crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
use worth_query_declaration::facade::application_schema::RelationDirection;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(super) fn admit_component_members<Entity>(
        &mut self,
        layout: &ComponentSetLayout,
        aggregate: &AggregateObservation,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        subject: &str,
    ) -> Result<(), Denial> {
        for member in aggregate.native().members() {
            AggregateObservation::checkpoint(context)
                .map_err(|cause| self.walk_stop(cause, subject))?;
            self.endpoint::<Entity>(self.execution, member, layout.member_kind, &layout.member)?;
            self.realized_scope.record(member);
            for (name, kind) in layout.sources.iter() {
                for direction in [RelationDirection::Forward, RelationDirection::Reverse] {
                    let read = self.read_adjacency(name, member, *kind, direction)?;
                    for edge in read.into_records() {
                        AggregateObservation::checkpoint(context)
                            .map_err(|cause| self.walk_stop(cause, subject))?;
                        if !aggregate.native().contains(edge.source)
                            || !aggregate.native().contains(edge.target)
                        {
                            return Err(Denial::new(Kind::InvalidComponent, subject));
                        }
                        let endpoint = match direction {
                            RelationDirection::Forward => edge.target,
                            RelationDirection::Reverse => edge.source,
                        };
                        self.endpoint::<Entity>(
                            self.execution,
                            endpoint,
                            layout.member_kind,
                            &layout.member,
                        )?;
                        self.realized_scope
                            .record_relation(edge.source, edge.target);
                    }
                }
            }
        }
        AggregateObservation::checkpoint(context).map_err(|cause| self.walk_stop(cause, subject))
    }
}
