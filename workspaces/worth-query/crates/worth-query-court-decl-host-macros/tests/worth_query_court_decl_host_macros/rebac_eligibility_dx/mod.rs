//! W9A-3: platform ReBAC "who may act?" on a QMS-shaped site.
//!
//! Fact-free eligibility answers from the same installed policy decision
//! admission uses, and never commits; a runtime principal registry onboards
//! users with a structurally unique identity; ability-gated pages re-admit
//! the reader on every page.
mod access;
mod eligibility_court;
mod paging_court;
mod program;
mod registry_court;
mod roles;
mod schema;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use program::{principal, QmsHome, SITE};
use schema::*;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::eligibility::{
    WorthQueryAbilityEligibility, WorthQueryEligibilityQuestion,
};
use worth_query_host::facade::{WorthQueryCommitWatchGuard, WorthQueryHostSession};

type Session<'home> = WorthQueryHostSession<'home, QmsSchema, program::QmsProgram>;
type ApproveQuestion = WorthQueryEligibilityQuestion<QmsSchema, Qms, Approve, DocumentId>;

fn temp_home_path(label: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("rebac-qms-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn session<'home>(
    home: &'home QmsHome,
    clock: &WorthQueryAdmissionClock,
    subject: &str,
) -> Session<'home> {
    home.session(principal(home, clock, subject))
}

/// "May this principal approve `document`?"
fn approve(document: &str) -> ApproveQuestion {
    WorthQueryEligibilityQuestion::new(
        Qms,
        Approve::reference(),
        DocumentFacts::document_id(),
        document.to_owned(),
    )
}

fn verdict(session: &Session<'_>, document: &str) -> WorthQueryAbilityEligibility {
    session
        .eligibility(&approve(document))
        .expect("an approval eligibility question is answered")
}

/// Whether approving `document` as this session commits.
fn approves(session: &Session<'_>, document: &str) -> bool {
    session
        .run(ApproveDocument {
            document: document.to_owned(),
            approved: true,
        })
        .is_ok()
}

/// Counts the Home's commits from now until the guard drops.
fn count_commits(home: &QmsHome) -> (Arc<AtomicUsize>, WorthQueryCommitWatchGuard) {
    let commits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&commits);
    let guard = home
        .on_commit(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .expect("commit watch registers");
    (commits, guard)
}

fn commits(counter: &AtomicUsize) -> usize {
    counter.load(Ordering::SeqCst)
}

/// The site administrator grants `operator` a document `role`.
fn assign(admin: &Session<'_>, assignment: &str, operator: u64, role: &str, document: &str) {
    admin
        .run(roles::AssignDocumentRole {
            site: SITE.to_owned(),
            assignment: assignment.to_owned(),
            operator,
            role: role.to_owned(),
            document: document.to_owned(),
        })
        .expect("the site administrator assigns a document role");
}

fn revoke(admin: &Session<'_>, assignment: &str) {
    admin
        .run(roles::RevokeAssignment {
            site: SITE.to_owned(),
            assignment: assignment.to_owned(),
        })
        .expect("the site administrator revokes an assignment");
}
