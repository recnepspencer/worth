use super::CurrentProjection;
use syn::visit::Visit;

// Syntax fixtures exercise the actual architecture rule, not Store behavior.
const CURRENT: &str = r#"
const CURRENT_RECOVERY_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v15";
struct PersistedPhysicalRecoveryProjection {
    source_root_generation: u64,
    operation: PersistedPhysicalRecoveryOperation,
}
impl PersistedPhysicalRecoveryProjection {
    fn encode(&self) -> Vec<u8> {
        let mut target = Vec::new();
        field(&mut target, CURRENT_RECOVERY_PROJECTION_DOMAIN);
        match &self.operation { Operation::Append(binding) => write(binding), Operation::Drop(binding) => write(binding) }
        target
    }
    fn decode_payload(bytes: &[u8]) -> Result<Self, Denial> {
        let mut cursor = Cursor::new(bytes);
        require_current_domain(cursor.field()?)?;
        decode_operation(cursor.byte()?)
    }
}
fn require_current_domain(domain: &[u8]) -> Result<(), Denial> {
    if domain == CURRENT_RECOVERY_PROJECTION_DOMAIN { return Ok(()); }
    let suffix = domain.strip_prefix(b"store.physical.recovery-projection.v").ok_or(Denial::Malformed)?;
    let version = suffix.iter().try_fold(0_u16, parse_digit).ok_or(Denial::Malformed)?;
    Err(Denial::UnsupportedVersion(version))
}
"#;

fn check(source: &str) -> Vec<String> {
    let file = syn::parse_file(source).expect("valid Rust syntax fixture");
    let mut rule = CurrentProjection {
        projection_scope: true,
        ..Default::default()
    };
    rule.visit_file(&file);
    rule.finish()
}

#[test]
fn current_domain_typed_operations_and_rejection_only_numeric_parser_are_allowed() {
    assert!(check(CURRENT).is_empty());
    let with_test = format!("{CURRENT}\n#[cfg(test)] fn old() {{ let _ = b\"store.physical.recovery-projection.v6\"; }}");
    assert!(check(&with_test).is_empty());
}

#[test]
fn a_second_grammar_is_denied_in_new_constants_and_macros() {
    for addition in [
        "const RENAMED_OLD: &[u8] = b\"store.physical.recovery-projection.v6\";",
        "macro_rules! abandoned { () => { b\"store.physical.recovery-projection.v14\" }; }",
    ] {
        assert!(check(&format!("{CURRENT}\n{addition}"))
            .iter()
            .any(|finding| finding.contains("domain literal")));
    }
}

#[test]
fn renamed_instance_selector_and_parallel_attachment_are_denied() {
    for field in [
        "wire_dialect: u8,",
        "release_head_effect: Option<Effect>,",
        "pub operation_tag: u8,",
    ] {
        let source = CURRENT.replace(
            "source_root_generation: u64,",
            &format!("source_root_generation: u64, {field}"),
        );
        assert!(check(&source)
            .iter()
            .any(|finding| finding.contains("fields must remain sealed")));
    }
    assert!(!check(&CURRENT.replace(
        "operation: PersistedPhysicalRecoveryOperation",
        "operation: u8"
    ))
    .is_empty());
}

#[test]
fn selector_encoder_and_late_domain_admission_are_denied() {
    let source = CURRENT.replace("field(&mut target, CURRENT_RECOVERY_PROJECTION_DOMAIN);", "field(&mut target, match operation { Op::Append => CURRENT_RECOVERY_PROJECTION_DOMAIN, Op::Drop => previous_domain });");
    assert!(check(&source)
        .iter()
        .any(|finding| finding.contains("encoder must write")));
    let source = CURRENT.replace(
        "require_current_domain(cursor.field()?)?;",
        "let body = allocate_body(); require_current_domain(cursor.field()?)?;",
    );
    assert!(check(&source)
        .iter()
        .any(|finding| finding.contains("before body decoding")));
}

#[test]
fn owning_buffer_and_allocation_free_cursor_cannot_be_counterfeited() {
    for source in [
        CURRENT.replace("Vec::new()", "allocate_body_then_target()"),
        CURRENT.replace("field(&mut target,", "field(&mut decoy,"),
        CURRENT.replace("        target\n", "        other_buffer\n"),
    ] {
        assert!(check(&source)
            .iter()
            .any(|finding| finding.contains("encoder must write")));
    }
    for source in [
        CURRENT.replace("Cursor::new(bytes)", "allocating_body_then_cursor(bytes)"),
        CURRENT.replace("Cursor::new(bytes)", "Cursor::new(allocated_bytes())"),
        CURRENT.replace(
            "require_current_domain(cursor.field()?)",
            "require_current_domain(decoy.field()?)",
        ),
    ] {
        assert!(check(&source)
            .iter()
            .any(|finding| finding.contains("before body decoding")));
    }
}

#[test]
fn fallback_success_or_domain_branch_relabeling_is_denied() {
    for source in [
        CURRENT.replace(
            "Err(Denial::UnsupportedVersion(version))",
            "if version == 6 { return Ok(()); } Err(Denial::UnsupportedVersion(version))",
        ),
        CURRENT.replace(
            "domain == CURRENT_RECOVERY_PROJECTION_DOMAIN",
            "domain == CURRENT_RECOVERY_PROJECTION_DOMAIN || domain == old_domain",
        ),
        CURRENT.replace(
            "domain == CURRENT_RECOVERY_PROJECTION_DOMAIN",
            "domain != CURRENT_RECOVERY_PROJECTION_DOMAIN",
        ),
        CURRENT.replace("return Ok(());", "return fallback_reader(domain);"),
        CURRENT.replace(
            "Err(Denial::UnsupportedVersion(version))",
            "return fallback_reader(domain);",
        ),
    ] {
        assert!(
            check(&source)
                .iter()
                .any(|finding| finding.contains("exactly one success")),
            "accepted {source}"
        );
    }
}

#[test]
fn missing_wire_owner_or_computed_identity_fails_closed() {
    for source in [
        CURRENT.replace("fn encode(", "fn renamed_encode("),
        CURRENT.replace("fn decode_payload(", "fn bypass_decode("),
        CURRENT.replace("fn require_current_domain(", "fn other_admission("),
        CURRENT.replace(
            "b\"store.physical.recovery-projection.v15\"",
            "choose_domain(operation)",
        ),
        CURRENT.replace("PersistedPhysicalRecoveryProjection", "OtherProjection"),
    ] {
        assert!(!check(&source).is_empty(), "accepted {source}");
    }
}

#[test]
fn semantic_codecs_cannot_take_a_wire_version_parameter() {
    let source = format!("{CURRENT}\nfn decode_operation(bytes: &[u8], version: u16) {{ select_operation(version, bytes); }}");
    assert!(check(&source)
        .iter()
        .any(|finding| finding.contains("inputs may not select")));
}
