use crate::planning::AccessPlanSelectionView;
use worth_store_contracts::DurableArtifactFamilyId;

use super::{
    denial::{map_security_denial, map_selection_denial, BTreeReplayDenied},
    request::BTreeReplayRequest,
};

pub(super) fn admit(
    request: &BTreeReplayRequest<'_>,
    physical_source: &super::AdmittedBTreeReplayPhysicalSource,
) -> Result<crate::BaselineBTreeReplayAdmission, BTreeReplayDenied> {
    let declarations = crate::layout_declarations();
    let declaration = declarations
        .declaration(DurableArtifactFamilyId::PhysicalPage)
        .expect("the permanent layout registry declares physical pages");
    let family = declarations
        .admit_physical_artifact_family(declaration, request.security)
        .into_result()
        .expect("the permanent physical-page declaration has production authority");
    let key_domain = declarations
        .admit_physical_key_domain(family, request.security)
        .into_result()
        .map_err(map_security_denial)?;
    let concrete_key = declarations
        .admit_page_key(key_domain, request.segment, request.page)
        .expect("typed nonzero page coordinates satisfy the admitted page domain");
    let materialization = crate::access_planning()
        .admit_btree_replay_materialization(family, request.catalog, physical_source)
        .into_result()
        .expect("an admitted replay source establishes exact replay coverage");
    let shape = crate::access_planning()
        .rebuild_access(crate::AccessLaneClassification::Maintenance)
        .expect("maintenance owns the declared rebuild-read shape");
    let admitted = crate::AccessPlanSelector
        .admit_recovery_request(family, concrete_key, materialization, shape)
        .expect("typed replay inputs form the canonical recovery request");
    let outcome = crate::AccessPlanSelector.select_admitted_with_budget(admitted, request.budget);
    match outcome.view() {
        AccessPlanSelectionView::BTreeReplayRecovery(_) => {
            Ok(crate::BaselineBTreeReplayAdmission::admit(
                outcome
                    .into_btree_replay_recovery()
                    .expect("view established B-tree replay selection"),
            ))
        }
        AccessPlanSelectionView::Denied(denial) => Err(map_selection_denial(denial.clone())),
        _ => unreachable!("an admitted B-tree replay request selects only B-tree replay"),
    }
}
