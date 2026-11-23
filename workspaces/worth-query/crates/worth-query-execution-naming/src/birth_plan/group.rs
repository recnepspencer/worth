//! Execution-time group birth and transition lowering onto Relational's
//! `AggregateGroupNamingAuthority`.

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_relational::facade::aggregate_identity::{
    AggregateGroupMembers, AggregateGroupNamingAuthority, AggregateGroupTransition,
    AggregateIdentity, AggregatePriorGroup,
};
use worth_relational::facade::lineage::PersistentNameIdentity;
use worth_relational::facade::schema::NativeMemberKey;

use crate::admission::admit_group_assignment;
use crate::denial::ExecutionNamingDenial;
use crate::lowering::LoweredExecutionGroupScheme;

/// One member participating in a proposed group partition.
#[derive(Clone, Debug)]
pub struct ExecutionGroupMemberSpec {
    pub member_identity: PersistentNameIdentity,
    pub native_key: NativeMemberKey,
    pub declared_role: String,
}

/// Admitted outcome of executing one Relational group naming transition.
#[derive(Clone, Debug)]
pub struct ExecutionGroupTransitionOutcome<G> {
    pub transition: AggregateGroupTransition,
    pub admitted_group_names: Vec<WorthQueryName<G>>,
}

impl LoweredExecutionGroupScheme {
    /// Evaluates one deterministic, allocation-independent group birth /
    /// split / merge / revival transition through Relational's
    /// [`AggregateGroupNamingAuthority`] and admits the minted group identities
    /// into kind-typed [`WorthQueryName<G>`].
    pub fn execute_group_transition<G>(
        &self,
        prior_groups: &[AggregatePriorGroup],
        retired_groups: &BTreeSet<AggregateIdentity>,
        proposed_partitions: &[Vec<ExecutionGroupMemberSpec>],
    ) -> Result<ExecutionGroupTransitionOutcome<G>, ExecutionNamingDenial> {
        let mut relational_partitions = Vec::with_capacity(proposed_partitions.len());
        for partition in proposed_partitions {
            let mut members = AggregateGroupMembers::new();
            for spec in partition {
                let role = self
                    .relational_member_role(&spec.declared_role)
                    .ok_or_else(|| ExecutionNamingDenial::UndeclaredRole {
                        scheme_id: self.scheme_id().to_owned(),
                        role_path: spec.declared_role.clone(),
                    })?;
                members
                    .insert(spec.member_identity, spec.native_key.clone(), role)
                    .map_err(ExecutionNamingDenial::GroupNaming)?;
            }
            relational_partitions.push(members);
        }
        let transition = AggregateGroupNamingAuthority::transition(
            self.relational_scheme(),
            prior_groups,
            retired_groups,
            relational_partitions,
        )
        .map_err(ExecutionNamingDenial::GroupNaming)?;
        let admitted_group_names = transition
            .assignments
            .iter()
            .map(admit_group_assignment::<G>)
            .collect();
        Ok(ExecutionGroupTransitionOutcome {
            transition,
            admitted_group_names,
        })
    }
}
