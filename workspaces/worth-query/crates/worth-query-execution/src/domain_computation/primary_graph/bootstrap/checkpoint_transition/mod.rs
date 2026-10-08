//! Native program transition before any World or ordinary writer exists.
mod authoring;
mod publication;
mod repair;
mod selection;
use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
pub use authoring::WorthQueryCheckpointMigrationWriter;
pub(in crate::domain_computation::primary_graph) use publication::transition_checkpoint;
pub use repair::WorthQueryCheckpointTransitionRecovery;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Exact descriptive rendering of the predecessor stored in the recovered image.
/// This value never becomes an admitted program revision or roster member.
#[derive(Clone, Debug)]
pub struct WorthQueryCheckpointProgramPredecessor(String);
impl WorthQueryCheckpointProgramPredecessor {
    pub fn new(rendering: &str) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        if rendering.len() != 64
            || !rendering
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(denial("checkpoint program predecessor requires exactly 64 lowercase hexadecimal characters"));
        }
        Ok(Self(rendering.to_owned()))
    }
}

/// Finite reconstructive bounds, additional to native candidate/publication limits.
/// Selection uses native examined-slot/materialization work. Authoring charges
/// one unit per row and entity field, plus the seed's inline storage, key/field
/// vector capacities and owned value allocations before lowering. Native limits
/// separately govern the lowered transaction overlay and prepared candidate.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryCheckpointTransitionResources {
    maximum_selection_work: usize,
    maximum_authored_units: usize,
    maximum_authored_bytes: usize,
}
impl WorthQueryCheckpointTransitionResources {
    pub fn bounded(
        maximum_selection_work: usize,
        maximum_authored_units: usize,
        maximum_authored_bytes: usize,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        if [
            maximum_selection_work,
            maximum_authored_units,
            maximum_authored_bytes,
        ]
        .into_iter()
        .any(|bound| bound == 0 || bound == usize::MAX)
        {
            return Err(denial(
                "checkpoint transition resources must be finite and nonzero",
            ));
        }
        Ok(Self {
            maximum_selection_work,
            maximum_authored_units,
            maximum_authored_bytes,
        })
    }
}

pub(in crate::domain_computation::primary_graph) struct CheckpointTransition<
    'authoring,
    'authority,
    Schema,
> {
    pub(in crate::domain_computation::primary_graph) recovery: std::rc::Rc<
        std::cell::RefCell<
            Option<crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint>,
        >,
    >,
    pub(in crate::domain_computation::primary_graph) predecessor:
        WorthQueryCheckpointProgramPredecessor,
    pub(in crate::domain_computation::primary_graph) resources:
        WorthQueryCheckpointTransitionResources,
    pub(in crate::domain_computation::primary_graph) capture_policy:
        crate::domain_computation::primary_graph::WorthQueryCheckpointCapturePolicy<
            'authoring,
            'authority,
        >,
    pub(in crate::domain_computation::primary_graph) author: Box<
        dyn FnOnce(
                &mut WorthQueryCheckpointMigrationWriter<'_, Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'authoring,
    >,
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
