//! Actual compiled successor with changed codec meaning but the old aspect revision.
use super::{model, wire_model};
#[path = "../../canonical_value/binding.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod binding;
#[path = "../derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod derived_query;
#[path = "../edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod edit;
#[path = "../host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod host;
#[path = "output.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod output;
#[path = "../producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod producer;
#[path = "../program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod program;
#[path = "../../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision discriminator requires real declarations and handlers compiled against its separate schema"
)]
mod query;
#[path = "../readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the missing-revision twin requires readiness bound to its own actual schema and producer"
)]
mod readiness;
#[path = "missing_revision_schema.rs"]
mod schema;

pub fn assert_missing_revision_refused(
    checkpoint: worth_query_host::facade::application_installation::WorthQueryApplicationCheckpoint,
) {
    use worth_query_host::facade::application_installation::in_memory_program_from_checkpoint;
    output::reset_decodes();
    let refusal = in_memory_program_from_checkpoint(
        host::program(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        host::limits(),
        checkpoint,
    )
    .err()
    .expect("changed codec meaning requires an actual declared aspect revision");
    assert_eq!(output::decodes(), 0);
    let worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::Graph(refusal) = refusal else {
        panic!("expected native checkpoint graph refusal: {refusal:?}");
    };
    assert_eq!(refusal.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
    assert!(
        refusal.subject().contains(
            "recovered field codec meaning changed without an advanced semantic aspect revision"
        ),
        "unexpected missing-revision refusal: {refusal:?}"
    );
}
