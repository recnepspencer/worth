//! Complete logical cohorts consume their actual source before Native identity.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::{member_work::checkpoint_names, AggregateObservation},
    invariant_projection::WorthQueryInvariantGraphComponent as Component,
};
use worth_query_declaration::facade::application_schema::ApplicationComponentSetRef;
use worth_relational::facade::{
    runtime::RelationalPinnedReadAdmissionDenial, schema::NativeMemberKey,
};

impl<S: ApplicationSchema> Reader<'_, S> {
    /// Read a complete logical component under this installation-owned projection.
    /// The answer retains genuine source and aggregate custody, without issuing
    /// operation decision facts or admitting a different projection's selectors.
    pub fn graph_component_of<Set, R, M>(
        &mut self,
        set: ApplicationComponentSetRef<S, Set, M>,
        relation: ApplicationDerivedRelationRef<S, R, M, M>,
        member: &Member<S, M>,
    ) -> Result<Component<S, Set, M>, Denial> {
        let denied = || Denial::new(Kind::ForeignIdentity, set.name());
        let (from, to) = self.projection_member_domains(relation)?;
        if from != to {
            return Err(denied());
        }
        member.check(self, &from)?;
        let installed = self.layout;
        let layout = installed
            .component_set(set.name())
            .ok_or_else(|| Denial::new(Kind::ComponentSetNotInstalled, set.name()))?;
        let source = installed
            .derived_relation(relation.name())
            .ok_or_else(denied)?;
        if layout.source.as_ref() != Some(&source.handle)
            || layout.member.as_ref() != set.member_selector().name()
            || installed.member_binding(set.member_selector()).as_ref() != Some(&from)
        {
            return Err(denied());
        }
        // Preserve positive and empty physical support, not just equal endpoints.
        let (native, allocation) =
            self.read_logical_source(relation, member.key.clone(), Shape::Component)?;
        let pin = self
            .runtime
            .read_truth()
            .pin_read_capability(self.snapshot, &[], self.execution)
            .map_err(|cause| match cause {
                RelationalPinnedReadAdmissionDenial::Request(stop) => self.walk_stop(
                    worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                    set.name(),
                ),
                _ => Denial::new(Kind::EndpointUnavailable, set.name()),
            })?;
        let execution = *self.execution;
        let captured = execution.run_native_with_checkpoint(|context| {
            if !layout
                .matches_native_definition(&pin, || AggregateObservation::checkpoint(context))
                .map_err(|cause| self.walk_stop(cause, set.name()))?
            {
                return Err(denied());
            }
            storage::aggregate::<NativeMemberKey>(&execution, &allocation, set.name())
                .map_err(|cause| self.walk_stop(cause, set.name()))?;
            let aggregate = self.read_member_aggregate_identity(
                &pin,
                layout.member_kind,
                layout.definition,
                member.key.clone(),
                set.name(),
                &allocation,
            )?;
            if aggregate.native().len() != native.data().endpoints().len() {
                return Err(Denial::new(Kind::InvalidComponent, set.name()));
            }
            for (actual, expected) in aggregate.native().members().zip(native.data().endpoints()) {
                let mut poll = || {
                    AggregateObservation::checkpoint(context)
                        .map_err(|cause| self.walk_stop(cause, set.name()))
                };
                poll()?;
                checkpoint_names(actual, &mut poll)?;
                checkpoint_names(expected, &mut poll)?;
                if actual != expected {
                    return Err(Denial::new(Kind::InvalidComponent, set.name()));
                }
            }
            let mut members = storage::vector(&execution, &allocation, aggregate.native().len())
                .map_err(|cause| self.walk_stop(cause, set.name()))?;
            for key in aggregate.native().members() {
                AggregateObservation::checkpoint(context)
                    .map_err(|cause| self.walk_stop(cause, set.name()))?;
                members.push(Member::from_source(
                    self,
                    key,
                    &from,
                    &native,
                    Some(&allocation),
                )?);
            }
            AggregateObservation::checkpoint(context)
                .map_err(|cause| self.walk_stop(cause, set.name()))?;
            Ok(Component::from_projection(
                members,
                aggregate,
                self.binding_identity.clone(),
                allocation,
            ))
        });
        match captured {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(stop.into(), set.name())),
        }
    }
}
