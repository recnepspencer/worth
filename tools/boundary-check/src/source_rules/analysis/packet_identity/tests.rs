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
fn placement_reads_belong_to_scheduling_not_nested_naming() {
    let allowed = "fn identities(packets: &[Packet]) { packets.iter().enumerate().map(|(ordinal, _)| PartitionIdentity::new(ordinal + 1)); }
        fn schedule_query_packets(identities: Vec<PartitionIdentity>, lease: &Lease) { let width = lease.policy().budget().max_workers().get(); }
        #[cfg(test)] fn fixture(width: usize) {}";
    assert!(diagnostics(allowed).is_empty());
    let found =
        diagnostics("fn schedule_query_packets(width: usize) { fn identities(width: usize) {} }");
    assert_eq!(found.len(), 1, "nested naming has no schedule permission");
    assert_eq!(found[0].code(), DiagnosticCode::Bc7011PacketIdentity);
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
    let found = diagnostics("fn schedule_query_packets(width: usize) { use std::thread::available_parallelism; available_parallelism(); }");
    assert_eq!(
        found.len(),
        1,
        "the schedule reads its lease, never the machine"
    );
    assert_eq!(found[0].code(), DiagnosticCode::Bc7011PacketIdentity);
}

#[test]
fn the_naming_signature_has_only_the_canonical_partition_set() {
    let source = format!("{READER}query_execution/leased_map/packet_identity.rs");
    for (signature, count) in [
        ("fn identities(packets: Vec<PacketizedQueryWork>) {}", 0),
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

#[test]
fn neutral_helpers_and_preparation_packetizers_are_scanned_by_crate() {
    use std::fs;
    let root = std::env::temp_dir().join(format!("worth-packet-scope-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    for package in ["worth-execution", "worth-relational"] {
        let owner = root.join("crates").join(package);
        fs::create_dir_all(owner.join("src")).unwrap();
        fs::write(
            owner.join("Cargo.toml"),
            format!(
                r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2021"
"#
            ),
        )
        .unwrap();
        for (file,source) in [
            ("neutral", "fn lanes() -> usize { std::thread::available_parallelism().unwrap().get() }"),
            ("query_preparation_packetization", "fn lanes(budget: &Budget) -> usize { std::thread::available_parallelism().unwrap().get() }"),
            ("neutral", "use std::thread::available_parallelism as lanes; fn identities(packets: &[Packet]) { lanes(); }"),
            ("neutral", "fn lanes() -> usize { ExecutionAuthorityConfig::available_worker_count().get() }"),
        ] {
            fs::write(owner.join("src/lib.rs"),format!("mod {file};")).unwrap();
            fs::write(owner.join(format!("src/{file}.rs")),source).unwrap();
            let found=validate(&root).unwrap();
            assert_eq!(found.len(),1,"machine width has one owner, independent of the file name: {package} {source}");
            assert_eq!(found[0].code(),DiagnosticCode::Bc7011PacketIdentity);
            assert_eq!(found[0].code().as_str(),"BC7011_PACKET_IDENTITY");
        }
        fs::remove_dir_all(owner).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn machine_observation_is_legal_only_in_the_exact_authority_width_function() {
    let file=crate::source_syntax::parse_file("impl ExecutionAuthorityConfig { pub fn available_worker_count() { std::thread::available_parallelism(); } }").unwrap();
    assert!(findings(WIDTH_SOURCE, &file.items).is_empty());
    for source in [
        "impl Other { fn available_worker_count() { std::thread::available_parallelism(); } }",
        "impl ExecutionAuthorityConfig { fn available_worker_count() { fn lanes() { std::thread::available_parallelism(); } } }",
    ] {
        let file=crate::source_syntax::parse_file(source).unwrap();
        let found=findings(WIDTH_SOURCE,&file.items);
        assert_eq!(found.len(),1,"an unrelated or nested function has no machine-observation permission");
        let diagnostic=Diagnostic::new(DiagnosticCode::Bc7011PacketIdentity,found[0].1.clone(),found[0].2);
        assert_eq!(diagnostic.code(),DiagnosticCode::Bc7011PacketIdentity);
    }
}

#[test]
fn a_second_crate_cannot_copy_the_authority_width_home() {
    let file=crate::source_syntax::parse_file("impl ExecutionAuthorityConfig { fn available_worker_count() { std::thread::available_parallelism(); } }").unwrap();
    let found = findings(
        "crates/worth-relational/src/authority/lease/worker_width.rs",
        &file.items,
    );
    assert_eq!(
        found.len(),
        1,
        "the machine observation has one crate and one function as its home"
    );
    let diagnostic = Diagnostic::new(
        DiagnosticCode::Bc7011PacketIdentity,
        found[0].1.clone(),
        found[0].2,
    );
    assert_eq!(diagnostic.code(), DiagnosticCode::Bc7011PacketIdentity);
}
