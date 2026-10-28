//! A real finite carrier/case universe; empty censuses still admit all source access.
use super::*;
use crate::graph_member::NativeMemberDomain;

impl AggregateReadPort<'_, '_> {
    pub fn all_projected_of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
    ) -> Result<AggregateReadCollection<NativeMemberKey>, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        let (state, execution, source) = self.view.aggregate_read_parts();
        let (source, committed) = source.ok_or(AggregateReadDenial::SourceUnavailable)?;
        budget
            .checkpoint(source.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = source
            .contract(AggregateSelection::Definition(definition), committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        let registered = self
            .require_source(contract, source.registry(committed))?
            .ok_or(AggregateReadDenial::WrongMemberKind)?;
        let NativeMemberDomain::Projected { projected } = &registered.parts().from else {
            return Err(AggregateReadDenial::WrongMemberKind);
        };
        let names = projected
            .name_bytes()
            .ok_or(AggregateReadDenial::PreparationRefused)?;
        let levels = u64::from(
            usize::BITS
                - source
                    .registry(committed)
                    .projected_member_domains
                    .len()
                    .max(1)
                    .leading_zeros(),
        );
        budget
            .checkpoint(
                (names as u64)
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(AggregateReadDenial::PreparationRefused)?,
            )
            .map_err(AggregateReadDenial::Budget)?;
        let domain = source
            .registry(committed)
            .projected_member_domain(projected)
            .ok_or(AggregateReadDenial::WrongMemberKind)?;
        self.view
            .require_entity_kind(domain.parts().carrier)
            .map_err(|cause| AggregateReadDenial::Budget(budget.refusal(cause)))?;
        let mut allowance = |units| budget.debit_native_rule_work(units);
        let (carriers, _retention) = funding::run_custom_read(&execution, &mut allowance, |work| {
            visible_members(state, domain.parts().carrier, work)
        })
        .map_err(|error| {
            source.latch_failure(error.clone());
            from_preparation(error)
        })?;
        let mut members = Vec::new();
        for carrier in carriers.0 {
            for case in &domain.parts().cases {
                budget
                    .checkpoint(case.len() as u64)
                    .map_err(AggregateReadDenial::Budget)?;
                let case = domain
                    .case(case)
                    .ok_or(AggregateReadDenial::WrongMemberKind)?;
                let observed = self
                    .view
                    .projected_member_from_carrier(&case, carrier.carrier())
                    .map_err(AggregateReadDenial::Derived)?;
                budget
                    .claim_retained(extent::vector_entry::<NativeMemberKey>())
                    .map_err(AggregateReadDenial::Budget)?;
                members.push(observed.key().clone());
            }
        }
        collection::read(self, source, definition, VisibleMembers(members))
    }
}
