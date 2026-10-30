//! Target laws must judge all recovered rows before a World exists.
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
pub use worth_query_host::facade::{application_installation as installation, primary_graph};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, publish_on_first_program, restore_on_first_program},
    presented_request::set_retention,
    programs::{validated_first_program, validated_second_program},
    readback::read_retention,
    schema::{Document, DocumentIdentityField, DocumentRetentionField, DocumentRetentionSchema},
    settled_verdict::{settle, RetentionVerdict},
};

// These courts judge target laws, not selection cost. Selection revalidates
// every recovered relation as well as every entity (3eb00a6dc4), so its work
// grows with the fixture's rows times its kinds; the ceiling is headroom.
pub const SELECTION_WORK: usize = 65_536;
