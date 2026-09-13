use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

use worth_store_formal_models::{
    map_replication_progress_outcome, map_replication_publication_outcome,
    map_replication_publication_readiness, map_replication_source_admission_outcome,
    ReplicationAdmissionAction,
};
use worth_store_physical_backend::BackendTargetProfile;
use worth_store_replication::{
    admit_replication_publication_readiness, admit_replication_source, AdmittedReplicationSource,
    ReplicationAdmissionRuntime, ReplicationCapsuleId, ReplicationPeerCapacity,
    ReplicationSourceDeclaration,
};
use worth_store_replication::{DurabilityReplayIdentity, DurabilityReplayKind};
#[cfg(test)]
use worth_store_replication::{ObserveReplicationAdmission, ReplicationAdmissionObservation};
use worth_store_security::{
    readmitted_foreign_wal_security_scope_for_test, readmitted_wal_security_scope_for_test,
    StoreReadmittedSecurityScope,
};

mod publication_denials;
use publication_denials::collect_publication_denials;

pub(in crate::courtroom::protocol_models) fn ordinary_replication_admission_actions(
) -> BTreeSet<ReplicationAdmissionAction> {
    let mut actions = BTreeSet::new();
    collect_source_admission_actions(&mut actions);

    let mut runtime = current_runtime();
    published_progress(
        &mut runtime,
        admitted_source(SourceSpec::standard(1, 10, 20)),
        &mut actions,
    );
    collect_progress_actions(&mut runtime, &mut actions);
    collect_publication_denials(&mut actions);
    actions
}

#[cfg(test)]
pub(in crate::courtroom::protocol_models::replication_admission) fn publication_pending_observation(
) -> ReplicationAdmissionObservation {
    let source = admitted_source(SourceSpec::standard(50, 40, 50));
    let runtime = current_runtime();
    let progress = runtime
        .observe_progress(source)
        .into_observed_progress()
        .unwrap();
    let readiness = admit_replication_publication_readiness(progress);
    let observation = readiness.observe_replication_admission();
    drop(readiness);
    observation
}

fn collect_source_admission_actions(actions: &mut BTreeSet<ReplicationAdmissionAction>) {
    let standard = SourceSpec::standard(10, 10, 20);
    let admitted = source_outcome(standard.clone(), false, false);
    actions.insert(map_replication_source_admission_outcome(&admitted));

    for spec in [
        SourceSpec {
            peer: "",
            ..standard.clone()
        },
        SourceSpec {
            epoch: 0,
            ..standard.clone()
        },
        SourceSpec {
            lineage: "",
            ..standard.clone()
        },
    ] {
        let outcome = source_outcome(spec, false, false);
        actions.insert(map_replication_source_admission_outcome(&outcome));
    }

    let wrong_authority = source_outcome(standard.clone(), true, false);
    actions.insert(map_replication_source_admission_outcome(&wrong_authority));
    let wrong_replay = source_outcome(standard, false, true);
    actions.insert(map_replication_source_admission_outcome(&wrong_replay));
}

fn collect_progress_actions(
    runtime: &mut ReplicationAdmissionRuntime,
    actions: &mut BTreeSet<ReplicationAdmissionAction>,
) {
    let fresh = runtime.observe_progress(admitted_source(SourceSpec {
        peer: "peer-b",
        ..SourceSpec::standard(20, 30, 40)
    }));
    actions.insert(map_replication_progress_outcome(&fresh));
    let fresh_progress = fresh.into_observed_progress().unwrap();
    let fresh_readiness = admit_replication_publication_readiness(fresh_progress);
    actions.insert(map_replication_publication_readiness(&fresh_readiness));
    let authority = fresh_readiness
        .source()
        .security_scope()
        .current_authority()
        .clone();
    let fresh_published = runtime.publish(fresh_readiness, &authority);
    actions.insert(map_replication_publication_outcome(&fresh_published));

    let resumed = runtime.observe_progress(admitted_source(SourceSpec::standard(21, 20, 30)));
    actions.insert(map_replication_progress_outcome(&resumed));
    let resumed_progress = resumed.into_observed_progress().unwrap();
    let resumed_readiness = admit_replication_publication_readiness(resumed_progress);
    actions.insert(map_replication_publication_readiness(&resumed_readiness));
    let authority = resumed_readiness
        .source()
        .security_scope()
        .current_authority()
        .clone();
    let resumed_published = runtime.publish(resumed_readiness, &authority);
    actions.insert(map_replication_publication_outcome(&resumed_published));

    for spec in [
        SourceSpec::standard(21, 20, 30),
        SourceSpec {
            scope: ScopeKind::Foreign,
            ..SourceSpec::standard(22, 30, 40)
        },
        SourceSpec {
            epoch: 8,
            ..SourceSpec::standard(23, 30, 40)
        },
        SourceSpec {
            lineage: "lineage-b",
            ..SourceSpec::standard(24, 30, 40)
        },
        SourceSpec::standard(25, 25, 35),
        SourceSpec::standard(26, 31, 40),
    ] {
        let outcome = runtime.observe_progress(admitted_source(spec));
        actions.insert(map_replication_progress_outcome(&outcome));
    }
}

fn published_progress(
    runtime: &mut ReplicationAdmissionRuntime,
    source: AdmittedReplicationSource,
    actions: &mut BTreeSet<ReplicationAdmissionAction>,
) {
    let progress_outcome = runtime.observe_progress(source);
    actions.insert(map_replication_progress_outcome(&progress_outcome));
    let progress = progress_outcome.into_observed_progress().unwrap();
    let readiness = admit_replication_publication_readiness(progress);
    actions.insert(map_replication_publication_readiness(&readiness));
    let authority = readiness
        .source()
        .security_scope()
        .current_authority()
        .clone();
    let publication = runtime.publish(readiness, &authority);
    actions.insert(map_replication_publication_outcome(&publication));
    publication.into_result().unwrap();
}

fn publication_readiness(
    runtime: &ReplicationAdmissionRuntime,
    source: AdmittedReplicationSource,
) -> worth_store_replication::ReplicationPublicationReadiness {
    let progress = runtime
        .observe_progress(source)
        .into_observed_progress()
        .unwrap();
    admit_replication_publication_readiness(progress)
}

fn current_runtime() -> ReplicationRuntimeFixture {
    open_runtime(ReplicationPeerCapacity::new(usize::MAX).unwrap())
}

fn open_runtime(capacity: ReplicationPeerCapacity) -> ReplicationRuntimeFixture {
    let scope = readmitted_wal_security_scope_for_test();
    let directory = worth_store_test_support::TemporaryDirectory::create("replication-protocol")
        .expect("replication progress directory");
    let runtime =
        ReplicationAdmissionRuntime::open(directory.path(), scope.current_authority(), capacity)
            .expect("replication runtime");
    ReplicationRuntimeFixture {
        runtime,
        _directory: directory,
    }
}

struct ReplicationRuntimeFixture {
    runtime: ReplicationAdmissionRuntime,
    _directory: worth_store_test_support::TemporaryDirectory,
}

impl Deref for ReplicationRuntimeFixture {
    type Target = ReplicationAdmissionRuntime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl DerefMut for ReplicationRuntimeFixture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}

fn admitted_source(spec: SourceSpec) -> AdmittedReplicationSource {
    source_outcome(spec, false, false).into_result().unwrap()
}

fn source_outcome(
    spec: SourceSpec,
    wrong_authority: bool,
    replay_mismatch: bool,
) -> worth_store_replication::ReplicationSourceAdmissionOutcome {
    let scope = scope(spec.scope);
    let declaration_digest = if replay_mismatch {
        "sha256:copied".to_owned()
    } else {
        spec.digest.clone()
    };
    let declaration = ReplicationSourceDeclaration::new(
        ReplicationCapsuleId(spec.capsule),
        spec.peer,
        spec.epoch,
        spec.lineage,
        declaration_digest,
        spec.first_lsn,
        spec.last_lsn,
    );
    let foreign = readmitted_foreign_wal_security_scope_for_test();
    let authority = if wrong_authority {
        foreign.current_authority().clone()
    } else {
        scope.current_authority().clone()
    };
    admit_replication_source(
        declaration,
        scope,
        &authority,
        replay_identity(spec.first_lsn, spec.last_lsn, &spec.digest),
    )
}

fn replay_identity(first_lsn: u64, last_lsn: u64, digest: &str) -> DurabilityReplayIdentity {
    DurabilityReplayIdentity::new(
        DurabilityReplayKind::WalFrame,
        BackendTargetProfile::PosixFileFsyncDirSync,
        digest,
        first_lsn,
        last_lsn,
    )
    .unwrap()
}

fn scope(kind: ScopeKind) -> StoreReadmittedSecurityScope {
    match kind {
        ScopeKind::Current => readmitted_wal_security_scope_for_test(),
        ScopeKind::Foreign => readmitted_foreign_wal_security_scope_for_test(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopeKind {
    Current,
    Foreign,
}

#[derive(Clone)]
struct SourceSpec {
    capsule: u64,
    peer: &'static str,
    epoch: u64,
    lineage: &'static str,
    first_lsn: u64,
    last_lsn: u64,
    digest: String,
    scope: ScopeKind,
}

impl SourceSpec {
    fn standard(capsule: u64, first_lsn: u64, last_lsn: u64) -> Self {
        Self {
            capsule,
            peer: "peer-a",
            epoch: 7,
            lineage: "lineage-a",
            first_lsn,
            last_lsn,
            digest: format!("sha256:replication-{capsule}"),
            scope: ScopeKind::Current,
        }
    }
}
