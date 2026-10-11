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

#[test]
fn module_level_machine_facts_cannot_hide_in_constants_or_macros() {
    for source in [
        "static WIDTH: std::sync::LazyLock<usize> = std::sync::LazyLock::new(|| std::thread::available_parallelism().unwrap().get());",
        "const MAX_WORKERS: usize = 4;",
        "const WIDTH: usize = 4;",
        "macro_rules! packet_names { () => { std::thread::available_parallelism() }; }",
    ] {
        let found = diagnostics(source);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code(), DiagnosticCode::Bc7011PacketIdentity);
    }
    assert!(diagnostics("fn schedule_query_packets(width: usize) { use std::thread::available_parallelism; available_parallelism(); }").is_empty());
}

#[test]
fn the_naming_signature_has_only_the_canonical_partition_set() {
    let source = format!("{READER}query_execution/leased_map/packet_identity.rs");
    for (signature, count) in [
        ("fn identities(packets: &[PacketizedQueryWork]) {}", 0),
        (
            "fn identities(packets: &[PacketizedQueryWork], hidden: usize) {}",
            1,
        ),
        ("fn identities(packets: &Lease) {}", 1),
        ("fn identities(packets: &mut [PacketizedQueryWork]) {}", 1),
    ] {
        let file = crate::source_syntax::parse_file(signature).unwrap();
        let found = findings(&source, &file.items);
        let diagnostics: Vec<_> = found
            .into_iter()
            .map(|(_, item, reason)| {
                Diagnostic::new(DiagnosticCode::Bc7011PacketIdentity, item, reason)
            })
            .collect();
        assert_eq!(diagnostics.len(), count);
        assert!(diagnostics
            .iter()
            .all(|d| d.code() == DiagnosticCode::Bc7011PacketIdentity));
    }
}
