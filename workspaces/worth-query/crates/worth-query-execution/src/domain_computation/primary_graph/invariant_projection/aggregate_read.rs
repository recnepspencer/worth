//! One exact Native aggregate acquisition and its original projection work debit.
use super::{
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::aggregate_observation::{
    AggregateObservation, AggregateQuery, AggregateSelection,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{
    PinnedAggregateReadDenial, PinnedAggregateReadDenialKind, RelationalPinnedReadAdmissionDenial,
};
impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(super) fn read_aggregate_identity(
        &mut self,
        entity_kind: worth_relational::facade::identity::KindId,
        selection: AggregateSelection,
        member: worth_relational::facade::identity::EntityId,
        subject: &str,
    ) -> Result<std::sync::Arc<AggregateObservation>, Denial> {
        let pin = self
            .runtime
            .read_truth()
            .pin_read_capability(self.snapshot, &[], self.execution)
            .map_err(|denial| match denial {
                RelationalPinnedReadAdmissionDenial::Request(stop) => self.walk_stop(
                    worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                    subject,
                ),
                _ => Denial::new(Kind::EndpointUnavailable, subject),
            })?;
        let result = AggregateObservation::read(
            &pin,
            self.execution,
            AggregateQuery {
                subject,
                entity_kind,
                selection,
                member,
                maximum_work: self.work_budget.remaining().map(|n| n as u64),
                control: None,
            },
        );
        self.finish_aggregate_read(result, selection, subject)
    }

    pub(super) fn read_member_aggregate_identity(
        &mut self,
        pin: &worth_relational::facade::runtime::RelationalPinnedRead,
        member_kind: worth_relational::facade::identity::KindId,
        definition: worth_foundational::facade::AggregateDefinitionIdentity,
        member: worth_relational::facade::schema::NativeMemberKey,
        subject: &str,
    ) -> Result<
        std::sync::Arc<AggregateObservation<worth_relational::facade::schema::NativeMemberKey>>,
        Denial,
    > {
        let result = AggregateObservation::read_member(
            pin,
            self.execution,
            subject,
            member_kind,
            definition,
            member,
            self.work_budget.remaining().map(|n| n as u64),
        );
        self.finish_aggregate_read(
            result,
            AggregateSelection::ComponentDefinition(definition),
            subject,
        )
    }

    fn finish_aggregate_read<Member>(
        &mut self,
        result: Result<std::sync::Arc<AggregateObservation<Member>>, PinnedAggregateReadDenial>,
        selection: AggregateSelection,
        subject: &str,
    ) -> Result<std::sync::Arc<AggregateObservation<Member>>, Denial> {
        let work = match &result {
            Ok(value) => value.native().work_units(),
            Err(denial) => denial.work,
        };
        if let Err(PinnedAggregateReadDenial {
            kind: PinnedAggregateReadDenialKind::Read(cause),
            ..
        }) = &result
        {
            self.native_read_stop.get_or_insert(*cause);
        }
        // Debit actual work to the existing projection allowance. Private identity
        // acquisitions are not business endpoint/edge diagnostic observations.
        if !usize::try_from(work).is_ok_and(|work| self.work_budget.consume(work)) {
            self.work_budget.mark_exceeded();
            return Err(Denial::new(Kind::WorkBudgetExceeded, subject));
        }
        result.map_err(|denial| self.aggregate_denial(denial, selection, subject))
    }

    fn aggregate_denial(
        &mut self,
        denial: PinnedAggregateReadDenial,
        selection: AggregateSelection,
        subject: &str,
    ) -> Denial {
        use PinnedAggregateReadDenialKind as Native;
        let kind = match denial.kind {
            Native::Read(cause) => return self.walk_stop(cause, subject),
            Native::NotAggregateRelation => {
                if selection.orbit_relation().is_some() {
                    Kind::NotOrbitRelation
                } else {
                    Kind::AggregateDefinitionUnavailable
                }
            }
            Native::DefinitionUnavailable => Kind::AggregateDefinitionUnavailable,
            Native::MemberUnavailable | Native::WrongMemberKind => Kind::EndpointUnavailable,
            Native::WorkLimitExceeded => {
                self.work_budget.mark_exceeded();
                Kind::WorkBudgetExceeded
            }
            _ => {
                if selection.orbit_relation().is_some() {
                    Kind::InvalidOrbit
                } else {
                    Kind::InvalidComponent
                }
            }
        };
        Denial::new(kind, subject)
    }
}
