//! W9A-1: ReBAC authoring DX end to end. Roles, abilities and rules declared
//! through `WorthQueryVocabulary`, `WorthQueryAbilities`, `role_abilities`,
//! ordered and set predicates, principal-field conditions and
//! `actor_is_not_target_of` admit and refuse exactly as authored, through a
//! Directory reopen; malformed rules refuse at declaration with the authored
//! wording.
mod admission_court;
mod declaration_court;
mod policy;
mod program;
mod role_authority_court;
mod schema;

use std::path::PathBuf;

use program::{principal, RebacHome};
use schema::{CaseView, ReadCase};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;
use worth_query_host::facade::WorthQuerySessionRunDenial;

type RunDenial = WorthQuerySessionRunDenial<WorthQueryOperationBodyDenial>;

fn temp_home_path(label: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("rebac-dx-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

/// The case as committed, read by the manager (who may read every site-1 case).
fn case(home: &RebacHome, clock: &WorthQueryAdmissionClock, case: u64) -> CaseView {
    let mut rows = home
        .session(principal(home, clock, "manager"))
        .query(ReadCase { case })
        .expect("the manager reads the case");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}
