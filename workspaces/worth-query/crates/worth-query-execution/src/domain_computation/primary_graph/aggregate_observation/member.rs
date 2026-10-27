//! Tagged members share the same actual Native aggregate observation owner.
use super::*;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational::facade::{runtime::PinnedAggregateReadScope, schema::NativeMemberKey as Key};
impl AggregateObservation<Key> {
    pub(in crate::domain_computation::primary_graph) fn read_member(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        subject: &str,
        carrier_kind: KindId,
        definition: AggregateDefinitionIdentity,
        member: Key,
        maximum_work: Option<u64>,
        allocation: Option<&Arc<super::super::source_facts::SourceFactsAllocation>>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        let scope = PinnedAggregateReadScope::for_member_definition(definition, member.clone())
            .with_work_limit(maximum_work);
        let native = pin.read_projected_aggregate(execution, scope)?;
        Ok(Arc::new(Self {
            key: AggregateKey::for_member(subject, carrier_kind, definition, member),
            native,
            source: pin.snapshot().clone(),
            _allocation: allocation.cloned(),
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        super::member_work::checkpoint_names(self.key.member(), &mut poll)?;
        super::member_work::checkpoint_names(other.key.member(), &mut poll)?;
        if self.key != other.key
            || self.native.definition() != other.native.definition()
            || self.native.identity() != other.native.identity()
            || self.native.len() != other.native.len()
        {
            return Ok(false);
        }
        for (a, b) in self.native.members().zip(other.native.members()) {
            poll()?;
            super::member_work::checkpoint_names(a, &mut poll)?;
            super::member_work::checkpoint_names(b, &mut poll)?;
            if a != b {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
}
impl AggregateKey<Key> {
    fn for_member(
        subject: &str,
        entity_kind: KindId,
        definition: AggregateDefinitionIdentity,
        member: Key,
    ) -> Self {
        Self {
            subject: Arc::from(subject),
            entity_kind,
            selection: AggregateSelection::ComponentDefinition(definition),
            member,
            definition,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn member(&self) -> &Key {
        &self.member
    }
}

impl AggregateObservation<Key> {
    pub(in crate::domain_computation::primary_graph) fn remains_equal_at(
        &self, pin:&RelationalPinnedRead, execution:&ExecutionRequest<'_, '_>,
        context:&mut MapKernelContext<'_, '_>, control:Option<&dyn Fn()->Result<(),Cause>>,
    )->Result<bool,crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
        use worth_relational::facade::runtime::PinnedAggregateReadDenialKind as Denial;
        let mut poll = || {
            AggregateObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok::<(), Failure>(())
        };
        poll()?;
        let mut scope = PinnedAggregateReadScope::for_member_definition(
            self.key.definition(),
            self.key.member().clone(),
        );
        if let Some(control) = control {
            scope = scope.with_control(control);
        }
        let current = match pin.read_projected_aggregate(execution, scope) {
            Ok(current) => current,
            Err(error) => {
                return match error.kind {
                    Denial::Read(cause) => Err(Failure::Execution(cause)),
                    Denial::MemberUnavailable
                    | Denial::WrongMemberKind
                    | Denial::NotAggregateRelation
                    | Denial::DefinitionUnavailable => Ok(false),
                    _ => Err(Failure::Unavailable),
                }
            }
        };
        poll()?;
        if current.definition() != self.native.definition()
            || current.identity() != self.native.identity()
            || current.len() != self.native.len()
        {
            return Ok(false);
        }
        for (a, b) in self.native.members().zip(current.members()) {
            poll()?;
            super::member_work::checkpoint_names(a, &mut poll)?;
            super::member_work::checkpoint_names(b, &mut poll)?;
            if a != b {
                return Ok(false);
            }
        }
        // This answer exposes identity and complete membership, not incident lineage.
        poll()?;
        Ok(true)
    }
}
