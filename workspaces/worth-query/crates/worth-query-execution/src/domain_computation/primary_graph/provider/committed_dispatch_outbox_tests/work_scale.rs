use super::*;

#[test]
fn exact_outbox_owner_read_work_stays_local_across_unrelated_dispatch_history() {
    for population in [10_u64, 1_000] {
        let world = installed_authorization_world(true);
        let provider = &world.application.primary_provider;
        let mut last = None;
        for identity in 1..=population {
            last = Some(commit_record(provider, identity));
        }
        let (binding, commit, runtime_id) = last.expect("positive population");
        let observed = provider
            .observe_expected(&binding, &commit, runtime_id)
            .expect("latest exact committed outbox remains owner-readable");
        assert_eq!(observed.work().exact_commit_snapshots(), 1);
        assert_eq!(observed.work().canonical_version_probes(), 1);
        assert_eq!(observed.work().examined_index_entries(), 0);
        assert_eq!(observed.work().direct_record_probes(), 1);
        assert_eq!(observed.work().projected_records(), 1);
        assert_eq!(observed.work().reconstruction_requests(), 0);
    }
}
