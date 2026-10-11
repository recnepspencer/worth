use super::*;
use std::{fs, process::Command};

// Compile the actual packet value's declaration, not a public lookalike.
// The fixture supplies only an owner mint and a caller; no runtime is involved.
#[test]
fn a_sibling_cannot_rename_a_sealed_packet_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source=root.join("crates/worth-relational/src/visibility/materialization/read_records/reader/query_execution/leased_map/packet_identity.rs");
    let file = crate::source_syntax::parse_file(&fs::read_to_string(source).unwrap()).unwrap();
    let declaration = file
        .items
        .iter()
        .filter(|item| match item {
            syn::Item::Struct(s) => s.ident == "PacketIdentity",
            syn::Item::Impl(i) => {
                matches!(i.self_ty.as_ref(),syn::Type::Path(p) if p.path.is_ident("PacketIdentity"))
            }
            _ => false,
        })
        .map(|item| item.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let directory =
        std::env::temp_dir().join(format!("worth-packet-identity-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let owner=format!("mod owner {{ {declaration} pub(super) fn packet() -> PacketIdentity {{ PacketIdentity(1) }} }}");
    let cases=[
        ("valid", "fn rename(id: owner::PacketIdentity, _lanes: u64) -> owner::PacketIdentity { id } fn main() { let _ = rename(owner::packet(), 8); }", true),
        ("renamed", "fn rename(id: owner::PacketIdentity, lanes: u64) -> owner::PacketIdentity { owner::PacketIdentity(id.0 % lanes + 1) } fn by_constructor(lanes:u64) -> owner::PacketIdentity { owner::PacketIdentity::new(lanes) } fn by_conversion(lanes:u64) -> owner::PacketIdentity { lanes.into() } fn by_default() -> owner::PacketIdentity { Default::default() } fn by_arithmetic(id:owner::PacketIdentity,lanes:u64) -> owner::PacketIdentity { id + lanes } fn main() { let _ = rename(owner::packet(), 8); }", false),
    ];
    for (name, caller, valid) in cases {
        let input = directory.join(format!("{name}.rs"));
        fs::write(&input, format!("{owner}\n{caller}")).unwrap();
        let output = Command::new("rustc")
            .args([
                "--edition",
                "2021",
                "--crate-name",
                "packet_identity_boundary",
                "--emit",
                "metadata",
                "--error-format",
                "json",
            ])
            .arg(&input)
            .arg("-o")
            .arg(directory.join(format!("{name}.rmeta")))
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            valid,
            "only the identity owner can make a different packet name: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if !valid {
            let codes: Vec<_> = String::from_utf8_lossy(&output.stderr)
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .filter_map(|value| value["code"]["code"].as_str().map(str::to_owned))
                .collect();
            assert!(
                codes.contains(&"E0603".to_owned()),
                "the packet constructor is private: {codes:?}"
            );
            assert!(
                codes.contains(&"E0616".to_owned()),
                "the packet's value is private: {codes:?}"
            );
            assert_eq!(
                codes.iter().filter(|c| *c == "E0599").count(),
                1,
                "a packet has no public constructor: {codes:?}"
            );
            assert_eq!(
                codes.iter().filter(|c| *c == "E0277").count(),
                2,
                "a packet has no From or Default: {codes:?}"
            );
            assert_eq!(
                codes.iter().filter(|c| *c == "E0369").count(),
                1,
                "a packet has no arithmetic: {codes:?}"
            );
        }
    }
    fs::remove_dir_all(directory).unwrap();
}
