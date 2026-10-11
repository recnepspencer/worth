use super::*;

fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let file = crate::source_syntax::parse_file(source).unwrap();
    findings(&format!("{READER}{SCHEDULE_SOURCE}"), &file.items)
        .into_iter()
        .map(|(_, item, reason)| {
            Diagnostic::new(DiagnosticCode::Bc7011PacketIdentity, item, reason)
        })
        .collect()
}

#[test]
fn naming_cannot_bucket_by_any_lease_width_or_machine_count() {
    for source in [
        "fn identities(packets: &[Packet]) { let buckets = packets.len().min(max_workers + 8); }",
        "fn identities(packets: &[Packet]) { let buckets = packets.len().min(std::thread::available_parallelism().unwrap().get()); }",
        "fn identities(packets: &[Packet], width: usize) {}",
        "fn identities(packets: &[Packet], lease: &Lease) { let buckets = packets.len().min(lease.policy().budget().max_workers().get()); }",
        "use std::thread::available_parallelism as machine_count; fn identities(packets: &[Packet]) { machine_count(); }",
    ] {
        let found = diagnostics(source);
        assert_eq!(found.len(), 1, "packet naming cannot see placement: {source}");
        assert_eq!(found[0].code(), DiagnosticCode::Bc7011PacketIdentity);
    }
}

#[test]
fn naming_precedes_scheduling_and_only_naming_mints_identities() {
    let allowed = "fn identities(packets: &[Packet]) { packets.iter().enumerate().map(|(ordinal, _)| PartitionIdentity::new(ordinal + 1)); }
        fn schedule_query_packets(identities: Vec<PartitionIdentity>, lease: &Lease) { let width = lease.policy().budget().max_workers().get(); std::thread::available_parallelism(); }
        #[cfg(test)] fn fixture(width: usize) {}";
    assert!(diagnostics(allowed).is_empty());
    let found =
        diagnostics("fn schedule_query_packets(width: usize) { PartitionIdentity::new(width); }");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code(), DiagnosticCode::Bc7011PacketIdentity);
    let found =
        diagnostics("fn schedule_query_packets(width: usize) { fn identities(width: usize) {} }");
    assert_eq!(found.len(), 1, "nested naming has no schedule permission");
}

#[test]
fn scope_follows_both_packetization_and_packet_execution() {
    for suffix in [
        "query_packetization.rs",
        "query_packetization/targets.rs",
        "query_execution/leased_map.rs",
        "query_execution/leased_map/packet_identity.rs",
    ] {
        assert!(packetizes(&format!("{READER}{suffix}")));
    }
    assert!(
        !packetizes(&format!("{READER}query_execution/leased_scan.rs")),
        "record byte width is not worker width"
    );
}
