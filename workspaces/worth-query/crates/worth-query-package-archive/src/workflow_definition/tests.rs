use super::*;
use crate::binary_output::BinaryOutput;
use crate::compatibility::{
    WorthQueryPackageArchiveCompatibilityPosture, WorthQueryPackageArchiveProtocolLayer,
};
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;

const TERMINAL: u8 = 6;
const OPERATION: u8 = 1;
const CONTROL: u8 = 1;
const RETRY: u8 = 3;

/// Hand-built draft bytes: a header with `maximum_nodes`, then each record
/// the caller writes.
struct Draft {
    output: BinaryOutput,
}

impl Draft {
    fn new(version: u16, maximum_nodes: u16) -> Self {
        let mut output = BinaryOutput::with_capacity(128);
        output.raw_bytes(b"WQWD");
        output.u16(version);
        output.text("spec.v1");
        output.text("definition.v1");
        output.u16(maximum_nodes);
        output.u16(8);
        output.u16(4);
        output.u16(1);
        output.raw_bytes(&[1]);
        output.u32(1);
        output.u32(1);
        output.u32(1);
        output.u32(4096);
        output.u64(0);
        output.text("a");
        Self { output }
    }

    fn current() -> Self {
        Self::new(WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION, 8)
    }

    fn terminals(mut self, identities: &[&str]) -> Self {
        self.output.u32(count(identities.len()));
        for identity in identities {
            self.output.raw_bytes(&[TERMINAL]);
            self.output.text(identity);
        }
        self
    }

    fn control(mut self, source: &str, target: &str, outcome: u8) -> Self {
        self.output.u32(1);
        self.output.raw_bytes(&[CONTROL]);
        self.output.text(source);
        self.output.text(target);
        self.output.raw_bytes(&[outcome]);
        self
    }

    fn bytes(self) -> Vec<u8> {
        self.output.into_bytes()
    }
}

fn count(length: usize) -> u32 {
    u32::try_from(length).expect("test counts are small")
}

fn well_formed() -> Vec<u8> {
    Draft::current()
        .terminals(&["a", "b"])
        .control("a", "b", 0)
        .bytes()
}

fn refused(bytes: &[u8]) -> Kind {
    decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect_err("the draft bytes are refused")
        .kind()
}

#[test]
fn a_well_formed_draft_decodes_its_header_and_records() {
    let draft =
        decode_workflow_definition_draft(&well_formed(), WorthQueryPackageArchiveLimits::DEFAULT)
            .expect("well-formed draft bytes decode");
    assert_eq!(draft.spec(), "spec.v1");
    assert_eq!(draft.identity(), "definition.v1");
    assert_eq!(draft.limits().maximum_nodes(), 8);
    assert_eq!(draft.node_count(), 2);
    assert_eq!(draft.connection_count(), 1);
}

#[test]
fn every_truncation_is_refused_without_panicking() {
    let bytes = well_formed();
    for length in 0..bytes.len() {
        decode_workflow_definition_draft(&bytes[..length], WorthQueryPackageArchiveLimits::DEFAULT)
            .expect_err("a truncated draft never decodes");
    }
}

#[test]
fn foreign_magic_and_trailing_bytes_are_refused() {
    let mut bytes = well_formed();
    bytes[0] = b'X';
    assert_eq!(refused(&bytes), Kind::InvalidMagic);

    let mut bytes = well_formed();
    bytes.push(0);
    assert_eq!(refused(&bytes), Kind::TrailingBytes);
}

#[test]
fn unsupported_draft_versions_report_their_exact_layer() {
    for (version, posture) in [
        (0, WorthQueryPackageArchiveCompatibilityPosture::InvalidZero),
        (
            WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION + 1,
            WorthQueryPackageArchiveCompatibilityPosture::ExceedsWindow,
        ),
    ] {
        let bytes = Draft::new(version, 8).terminals(&["a"]).bytes();
        let denial =
            decode_workflow_definition_draft(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                .expect_err("an unsupported version is refused before its body");
        assert_eq!(
            denial.kind(),
            Kind::UnsupportedWorkflowDefinitionDraftVersion
        );
        let compatibility = denial.compatibility().expect("a version denial");
        assert_eq!(
            compatibility.layer(),
            WorthQueryPackageArchiveProtocolLayer::WorkflowDefinitionDraft
        );
        assert_eq!(compatibility.observed_version(), version);
        assert_eq!(compatibility.posture(), posture);
    }
}

#[test]
fn counts_are_bounded_by_declared_limits_and_remaining_bytes() {
    let over_limit = Draft::new(WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION, 1)
        .terminals(&["a", "b"])
        .control("a", "b", 0)
        .bytes();
    assert_eq!(refused(&over_limit), Kind::NestedEntryBudgetExceeded);

    let mut hostile = Draft::new(
        WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION,
        u16::MAX,
    );
    hostile.output.u32(u32::from(u16::MAX));
    assert_eq!(refused(&hostile.bytes()), Kind::Truncated);
}

#[test]
fn nodes_must_arrive_in_strictly_ascending_identity_order() {
    for identities in [["b", "a"], ["a", "a"]] {
        let bytes = Draft::current()
            .terminals(&identities)
            .control("a", "b", 0)
            .bytes();
        assert_eq!(refused(&bytes), Kind::NonCanonicalRecordSequence);
    }
}

#[test]
fn malformed_records_are_refused_by_kind() {
    let mut unknown_node = Draft::current();
    unknown_node.output.u32(1);
    unknown_node.output.raw_bytes(&[TERMINAL + 1]);
    unknown_node.output.text("a");
    assert_eq!(
        refused(&unknown_node.bytes()),
        Kind::UnsupportedRecordVariant
    );

    let unknown_outcome = Draft::current()
        .terminals(&["a", "b"])
        .control("a", "b", 9)
        .bytes();
    assert_eq!(refused(&unknown_outcome), Kind::UnsupportedRecordVariant);

    let mut invalid_boolean = Draft::current();
    invalid_boolean.output.u32(1);
    invalid_boolean.output.raw_bytes(&[OPERATION]);
    for field in ["a", "operation", "input", ""] {
        invalid_boolean.output.text(field);
    }
    invalid_boolean.output.raw_bytes(&[2]);
    assert_eq!(
        refused(&invalid_boolean.bytes()),
        Kind::InvalidBooleanEncoding
    );

    let empty_identity = Draft::current().terminals(&[""]).bytes();
    assert_eq!(refused(&empty_identity), Kind::InvalidRecordShape);

    let mut zero_attempts = Draft::current().terminals(&["a", "b"]);
    zero_attempts.output.u32(1);
    zero_attempts.output.raw_bytes(&[RETRY]);
    zero_attempts.output.text("a");
    zero_attempts.output.text("b");
    zero_attempts.output.raw_bytes(&[0]);
    zero_attempts.output.text("again");
    zero_attempts.output.u16(0);
    assert_eq!(refused(&zero_attempts.bytes()), Kind::InvalidRecordShape);
}

#[test]
fn zero_definition_limits_are_refused() {
    let bytes = Draft::new(WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION, 0)
        .terminals(&[])
        .bytes();
    assert_eq!(refused(&bytes), Kind::InvalidRecordShape);
}

#[test]
fn drafts_beyond_the_byte_budget_are_refused_before_decoding() {
    let limits = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_archive_bytes(8);
    let denial = decode_workflow_definition_draft(&well_formed(), limits)
        .expect_err("the draft exceeds the caller's budget");
    assert_eq!(denial.kind(), Kind::LogicalByteBudgetExceeded);
}
