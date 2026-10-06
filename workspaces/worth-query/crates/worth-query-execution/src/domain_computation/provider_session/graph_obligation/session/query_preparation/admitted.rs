//! Before-work admission for the already authorized Query session start.

use worth_query_admission::facade::graph_obligation::WorthQueryAdmittedGraphWorkPlan;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::TruthBranchIdentity;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisSelectionIdentity,
    WorthQueryPrimaryGraph,
};

use super::super::{
    WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession,
    WorthQueryManagedGraphWorkSessionStartDenial,
};
use super::WorthQueryPreparedQuerySessionIdentity;

pub(in crate::domain_computation) enum WorthQueryAdmittedQuerySessionStartStop<Stop> {
    Admission(Stop),
    WorkCounterOverflow,
    PreparationBytesCounterOverflow,
    Session(WorthQueryManagedGraphWorkSessionStartDenial),
}

impl WorthQueryManagedGraphWorkSession {
    /// Fund the real session copies, comparisons and temporary Bridge labels
    /// before the normal owner opens the session. `graph` issues the same
    /// private read port; no primary-index inventory is rebuilt for this read.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_query_with_reserved_identity_admitted<Stop>(
        reserved: WorthQueryPreparedQuerySessionIdentity,
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: &crate::basis::WorthQueryProductObservationLease,
        authorization_product: &crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        graph: &WorthQueryPrimaryGraph,
        mut prepare: impl FnMut(u64, usize) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryAdmittedQuerySessionStartStop<Stop>> {
        // The following preparation reads only fixed-width lengths and
        // selected identity axes. Fund those reads before calculating widths.
        prepare(24, 0).map_err(WorthQueryAdmittedQuerySessionStartStop::Admission)?;
        let selected_name = match basis.selection() {
            WorthQueryApplicationBasisSelectionIdentity::Product(identity) => {
                identity.branch_identity().name().as_str()
            }
            WorthQueryApplicationBasisSelectionIdentity::Relational => {
                return Err(WorthQueryAdmittedQuerySessionStartStop::Session(
                    WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch,
                ));
            }
        };
        let basis_branch = basis.branch_id().0.as_str();
        let authorization_branch = authorization_product
            .relational_basis_descriptor()
            .branch_id()
            .0
            .as_str();
        let descriptor_branch = basis.descriptor().branch_id().0.as_str();
        let reference_branch = basis.descriptor().reference().branch_id().as_str();
        let parent_commits = basis
            .descriptor()
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let carried_name = product.observation().branch_identity().name().as_str();
        let (work, bytes) = session_copy_requirements(
            basis_branch,
            authorization_branch,
            descriptor_branch,
            reference_branch,
            parent_commits,
            selected_name,
            carried_name,
            subject_authority,
            provider,
        )?;
        prepare(work, bytes).map_err(WorthQueryAdmittedQuerySessionStartStop::Admission)?;
        Self::start_query_with_reserved_identity(
            reserved,
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            basis,
            product.retained_clone(),
            authorization_product,
            provider,
            graph.query_session_port(),
        )
        .map_err(WorthQueryAdmittedQuerySessionStartStop::Session)
    }
}

fn session_copy_requirements<Stop>(
    basis_branch: &str,
    authorization_branch: &str,
    descriptor_branch: &str,
    reference_branch: &str,
    parent_commits: usize,
    selected_name: &str,
    carried_name: &str,
    subject_authority: &str,
    provider: &str,
) -> Result<(u64, usize), WorthQueryAdmittedQuerySessionStartStop<Stop>> {
    let work_overflow = || WorthQueryAdmittedQuerySessionStartStop::WorkCounterOverflow;
    let bytes_overflow =
        || WorthQueryAdmittedQuerySessionStartStop::PreparationBytesCounterOverflow;
    let mut work = 0usize;
    let mut bytes = 0usize;
    for branch in [basis_branch, authorization_branch] {
        let framed =
            TruthBranchIdentity::relational_branch_framed_len(branch).ok_or_else(work_overflow)?;
        // BranchId String, payload Arc<str>, exact framed String, final
        // framed Arc<str>. Each initialized byte is copied at most twice.
        let copied = branch
            .len()
            .checked_add(framed)
            .and_then(|value| value.checked_mul(2))
            .ok_or_else(work_overflow)?;
        work = work.checked_add(copied).ok_or_else(work_overflow)?;
        work = work
            .checked_add(2 * super::super::arc_str_layout::initialized_header_work())
            .ok_or_else(work_overflow)?;
        let arc_backing = super::super::arc_str_layout::backing_bytes(branch.len())
            .and_then(|raw| {
                super::super::arc_str_layout::backing_bytes(framed)
                    .and_then(|label| raw.checked_add(label))
            })
            .ok_or_else(bytes_overflow)?;
        // The two Strings own one raw branch and one framed label. The two
        // Arc allocations include their payloads in arc_backing already.
        bytes = bytes
            .checked_add(
                branch
                    .len()
                    .checked_add(framed)
                    .ok_or_else(bytes_overflow)?,
            )
            .and_then(|value| value.checked_add(arc_backing))
            .ok_or_else(bytes_overflow)?;
    }
    // The retained Query basis makes one descriptive branch and one native
    // descriptor copy; session identity owns the subject and provider text.
    for copied in [
        basis_branch.len(),
        descriptor_branch.len(),
        reference_branch.len(),
        subject_authority.len(),
        provider.len(),
    ] {
        work = work.checked_add(copied).ok_or_else(work_overflow)?;
        bytes = bytes.checked_add(copied).ok_or_else(bytes_overflow)?;
    }
    // The descriptive basis clone also duplicates the native reference's
    // parent-commit Vec. Its initialized IDs are copied once.
    work = work
        .checked_add(parent_commits)
        .and_then(|value| value.checked_add(7))
        .ok_or_else(work_overflow)?;
    bytes = bytes
        .checked_add(
            parent_commits
                .checked_mul(std::mem::size_of::<u64>())
                .ok_or_else(bytes_overflow)?,
        )
        .ok_or_else(bytes_overflow)?;
    // Two basis-branch equalities, one Product branch-name equality, the
    // binding/obligation digest comparisons and shallow owner/Arc visits.
    let product_name_comparison = selected_name
        .len()
        .max(carried_name.len())
        .checked_add(1)
        .ok_or_else(work_overflow)?;
    work = work
        .checked_add(
            basis_branch
                .len()
                .checked_add(1)
                .ok_or_else(work_overflow)?
                .checked_mul(2)
                .ok_or_else(work_overflow)?,
        )
        .and_then(|value| value.checked_add(product_name_comparison))
        .and_then(|value| value.checked_add(192 + 32))
        .ok_or_else(work_overflow)?;
    Ok((u64::try_from(work).map_err(|_| work_overflow())?, bytes))
}
