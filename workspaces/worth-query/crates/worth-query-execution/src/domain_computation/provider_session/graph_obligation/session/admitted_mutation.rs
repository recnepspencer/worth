//! Before-work preparation for a selected mutation session.

use worth_query_admission::facade::graph_obligation::WorthQueryAdmittedGraphWorkPlan;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::TruthBranchIdentity;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotLease;

use super::{WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession};

pub(in crate::domain_computation) enum WorthQueryAdmittedMutationSessionStartStop<Stop> {
    Admission(Stop),
    WorkCounterOverflow,
    PreparationBytesCounterOverflow,
    Session,
}

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation_admitted<Stop>(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        provider: &str,
        mut prepare: impl FnMut(u64, usize) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryAdmittedMutationSessionStartStop<Stop>> {
        use WorthQueryAdmittedMutationSessionStartStop as Refusal;
        // Header reads precede the variable initialized branch/text lengths.
        prepare(12, 0).map_err(Refusal::Admission)?;
        let branch = lease.snapshot().branch_id().0.as_str();
        let framed = TruthBranchIdentity::relational_branch_framed_len(branch)
            .ok_or(Refusal::WorkCounterOverflow)?;
        let arc_bytes = super::arc_str_layout::backing_bytes(branch.len())
            .and_then(|raw| {
                super::arc_str_layout::backing_bytes(framed)
                    .and_then(|label| raw.checked_add(label))
            })
            .ok_or(Refusal::PreparationBytesCounterOverflow)?;
        let branch_and_truth = branch
            .len()
            .checked_add(framed)
            .and_then(|width| width.checked_mul(2))
            .ok_or(Refusal::WorkCounterOverflow)?;
        let branch_copy = branch_and_truth
            .checked_add(branch.len())
            .ok_or(Refusal::WorkCounterOverflow)?;
        let text_copy = branch_copy
            .checked_add(subject_authority.len())
            .and_then(|width| width.checked_add(provider.len()))
            .ok_or(Refusal::WorkCounterOverflow)?;
        // String backing is two BranchIds, the framed label, and session
        // subject/provider. Arc backing below already includes its payloads.
        let string_backing = branch
            .len()
            .checked_mul(2)
            .and_then(|width| width.checked_add(framed))
            .and_then(|width| width.checked_add(subject_authority.len()))
            .and_then(|width| width.checked_add(provider.len()))
            .ok_or(Refusal::PreparationBytesCounterOverflow)?;
        let backing = string_backing
            .checked_add(arc_bytes)
            .ok_or(Refusal::PreparationBytesCounterOverflow)?;
        // The unchanged session checks binding and obligation at start and
        // again in retained affinity. Both operands are initialized values.
        let identity_copies_and_comparisons = 80usize + 32 + 2 * 2 * (80 + 32);
        let arc_initialization = 2 * super::arc_str_layout::initialized_header_work();
        let branch_comparisons = branch
            .len()
            .checked_mul(4)
            .ok_or(Refusal::WorkCounterOverflow)?;
        let work = text_copy
            .checked_add(identity_copies_and_comparisons)
            .and_then(|work| work.checked_add(branch_comparisons))
            .and_then(|work| work.checked_add(arc_initialization))
            .and_then(|work| work.checked_add(18))
            .ok_or(Refusal::WorkCounterOverflow)?;
        prepare(
            u64::try_from(work).map_err(|_| Refusal::WorkCounterOverflow)?,
            backing,
        )
        .map_err(Refusal::Admission)?;
        Self::start_mutation(
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            lease,
            provider,
        )
        .map_err(|_| Refusal::Session)
    }
}
