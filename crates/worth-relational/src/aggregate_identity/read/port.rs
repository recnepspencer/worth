//! Exact-source selection and original-funded aggregate acquisition.
use super::*;
use crate::aggregate_identity::selection::AggregateSelection;
use crate::graph_member::{NativeMemberDomain, NativeMemberKey as Member};
use crate::identity::data::KindId;
use crate::projected_member_domain::ObservedProjectedMember;
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};

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
            member.into(),
        )
        .map(AggregateRead::from_backing)
    }

    /// Read an explicitly installed aggregate definition, including a component set.
    pub fn of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
        member: EntityId,
    ) -> Result<AggregateRead, AggregateReadDenial> {
        self.read_selected(AggregateSelection::Definition(definition), member.into())
            .map(AggregateRead::from_backing)
    }

    /// Read a logical cohort only from the actual current view's observed member.
    pub fn of_projected_definition(
        &self,
        definition: AggregateDefinitionIdentity,
        member: &ObservedProjectedMember,
    ) -> Result<AggregateRead<Member>, AggregateReadDenial> {
        self.view
            .validate_projected_member(member)
            .map_err(AggregateReadDenial::Derived)?;
        self.read_selected(
            AggregateSelection::Definition(definition),
            member.key().clone(),
        )
        .map(AggregateRead::from_backing)
    }

    pub(super) fn read_selected(
        &self,
        selection: AggregateSelection,
        member: Member,
    ) -> Result<ReadBacking, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        let actual_kind = self
            .view
            .entity_kind(member.carrier())
            .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        let (state, execution, source) = self.view.aggregate_read_parts();
        let (source, committed) = source.ok_or(AggregateReadDenial::SourceUnavailable)?;
        budget
            .checkpoint(source.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = source
            .contract(selection, committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        let derived = self.require_source(contract, source.registry(committed))?;
        let domain = derived.as_ref().map(|definition| &definition.parts().from);
        let matches = match (domain, &member) {
            (None | Some(NativeMemberDomain::Entity(_)), Member::Entity(_)) => true,
            (Some(NativeMemberDomain::Projected { projected }), Member::Projected(member)) => {
                budget
                    .checkpoint(
                        member
                            .domain
                            .name_bytes()
                            .and_then(|bytes| u64::try_from(bytes).ok())
                            .ok_or(AggregateReadDenial::PreparationRefused)?,
                    )
                    .map_err(AggregateReadDenial::Budget)?;
                &member.domain == projected
            }
            _ => false,
        };
        if !matches {
            return Err(AggregateReadDenial::WrongMemberKind);
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
                budget
                    .checkpoint(
                        member
                            .comparison_work(entries)
                            .and_then(|units| u64::try_from(units).ok())
                            .ok_or(AggregateReadDenial::PreparationRefused)?,
                    )
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
                    return Ok(ReadBacking::Proposed {
                        projection,
                        definition: definition_id,
                        assignment,
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
            source.registry(committed),
            member,
            &mut allowance,
        )
        .map_err(|error| {
            source.latch_failure(error.clone());
            from_preparation(error)
        })?;
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        Ok(ReadBacking::Stored(stored))
    }
    pub(super) fn require_source(
        &self,
        contract: &AggregateIdentityContractDeclaration,
        registry: &RelationalSchemaRegistry,
    ) -> Result<Option<crate::derived_relation::DerivedRelationHandle>, AggregateReadDenial> {
        if let Some(identity) = contract.source.derived_relation() {
            let names = identity
                .schema
                .0
                .len()
                .checked_add(identity.name.len())
                .ok_or(AggregateReadDenial::PreparationRefused)?;
            let levels =
                u64::from(usize::BITS - registry.derived_relations.len().max(1).leading_zeros());
            self.view
                .read_budget()
                .checkpoint(
                    (names as u64)
                        .checked_mul(11)
                        .and_then(|units| units.checked_mul(levels))
                        .ok_or(AggregateReadDenial::PreparationRefused)?,
                )
                .map_err(AggregateReadDenial::Budget)?;
            let handle = registry
                .derived_relation(identity)
                .ok_or(AggregateReadDenial::NotAggregateRelation)?;
            self.view
                .require_derived_relation(&handle)
                .map_err(AggregateReadDenial::Derived)?;
            Ok(Some(handle))
        } else {
            for &kind in contract
                .source
                .stored_relations()
                .ok_or(AggregateReadDenial::NotAggregateRelation)?
            {
                self.view.require_relation_kind(kind).map_err(|error| {
                    AggregateReadDenial::Budget(self.view.read_budget().refusal(error))
                })?;
            }
            Ok(None)
        }
    }
}
