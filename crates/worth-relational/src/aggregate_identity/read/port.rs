//! Exact-source selection and original-funded aggregate acquisition.
use super::*;
use crate::aggregate_identity::selection::AggregateSelection;
use crate::identity::data::KindId;

impl<'view, 'runtime> AggregateReadPort<'view, 'runtime> {
    pub(crate) fn new(view: &'view StructuralRelationView<'runtime>) -> Self {
        Self { view }
    }

    /// Read one aggregate of the installed stored source relation. Shared
    /// proposal preparation is paid once by the original Native request. A
    /// committed/unaffected group costs O((v+a) log(v+a)) per deliberate read,
    /// debiting this rule's existing allowance and the request once each.
    pub fn of_source_relation(
        &self,
        relation: KindId,
        member: EntityId,
    ) -> Result<AggregateRead, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        self.view
            .require_relation_kind(relation)
            .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        self.read_selected(
            AggregateSelection::Orbit {
                relation,
                definition: None,
            },
            member,
        )
    }

    /// Read an explicitly installed aggregate definition, including a component set.
    pub fn of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
        member: EntityId,
    ) -> Result<AggregateRead, AggregateReadDenial> {
        self.read_selected(AggregateSelection::Definition(definition), member)
    }

    fn read_selected(
        &self,
        selection: AggregateSelection,
        member: EntityId,
    ) -> Result<AggregateRead, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        let actual_kind = self
            .view
            .entity_kind(member)
            .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        let (state, execution, source) = self.view.aggregate_read_parts();
        let (source, committed) = source.ok_or(AggregateReadDenial::SourceUnavailable)?;
        budget
            .checkpoint(source.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = source
            .contract(selection, committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        for &kind in contract.source.relations() {
            self.view
                .require_relation_kind(kind)
                .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        }
        let definition_id = contract.definition;
        if actual_kind != contract.member_kind {
            return Err(AggregateReadDenial::WrongMemberKind);
        }
        if !committed && source.is_logical_mutation() {
            let projection = source
                .projection_in_custom_scope(&execution)
                .map_err(from_preparation)?;
            let definition = projection
                .data
                .definitions
                .get(&definition_id)
                .ok_or_else(|| inconsistent(source, "prepared aggregate definition is absent"))?;
            if definition.contract.definition != contract.definition {
                return Err(inconsistent(
                    source,
                    "prepared aggregate definition affinity differs",
                ));
            }
            if let Some(changed) = &definition.changed {
                let entries = changed
                    .member_assignments
                    .len()
                    .saturating_add(changed.prior_memberships.len());
                budget
                    .checkpoint((usize::BITS - entries.max(1).leading_zeros()) as u64 * 22 + 1)
                    .map_err(AggregateReadDenial::Budget)?;
                if let Some(&assignment) = changed.member_assignments.get(&member) {
                    if changed
                        .transition
                        .assignments
                        .get(assignment)
                        .is_none_or(|group| !group.members.contains(&member))
                    {
                        return Err(inconsistent(
                            source,
                            "prepared member assignment differs from its transition",
                        ));
                    }
                    budget.check_live().map_err(AggregateReadDenial::Budget)?;
                    return Ok(AggregateRead {
                        backing: ReadBacking::Proposed {
                            projection,
                            definition: definition_id,
                            assignment,
                        },
                    });
                }
                if changed.prior_memberships.contains_key(&member) {
                    return Err(inconsistent(
                        source,
                        "affected proposed member has no current assignment",
                    ));
                }
            }
            // The complete actual affected-cohort proof excludes this member.
            // Only this proven-unaffected path may read its committed binding.
        }
        let mut allowance = |units| budget.debit_native_rule_work(units);
        let stored = super::super::stored_read::in_custom_scope(
            &execution,
            state,
            contract,
            member,
            &mut allowance,
        )
        .map_err(|error| {
            source.latch_failure(error.clone());
            from_preparation(error)
        })?;
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        Ok(AggregateRead {
            backing: ReadBacking::Stored(stored),
        })
    }
}
