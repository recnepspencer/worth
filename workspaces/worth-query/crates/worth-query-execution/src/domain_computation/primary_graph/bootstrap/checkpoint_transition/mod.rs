//! Native program transition before any World or ordinary writer exists.
mod authoring;
mod publication;
mod repair;
mod selection;
use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
pub use authoring::WorthQueryOpenAdoptionWriter;
pub(in crate::domain_computation::primary_graph) use publication::transition_checkpoint;
pub use repair::WorthQueryOpenAdoptionRecovery;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Exact descriptive rendering of the predecessor stored in the recovered image.
/// This value never becomes an admitted program revision or roster member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOpenAdoptionPredecessor(String);
impl WorthQueryOpenAdoptionPredecessor {
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

    /// The activation value a recovered image holds when it names this predecessor.
    fn rendering(&self) -> worth_foundational::facade::AspectValue {
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from(self.0.as_str()),
        )
    }
}

/// Finite reconstructive bounds, additional to native candidate/publication limits.
/// Selection uses native examined-slot/materialization work. Authoring charges
/// one unit per row and entity field, plus the seed's inline storage, key/field
/// vector capacities and owned value allocations before lowering. Native limits
/// separately govern the lowered transaction overlay and prepared candidate.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryOpenAdoptionResources {
    maximum_selection_work: usize,
    maximum_authored_units: usize,
    maximum_authored_bytes: usize,
}
impl WorthQueryOpenAdoptionResources {
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

/// One adoption declared for open. It runs only when the home's image names
/// `predecessor`; an empty home or a rostered image never runs it.
///
/// It stays separate from live branch adoption: the predecessor is a
/// descriptive rendering and the step runs before any World exists.
pub struct WorthQueryOpenAdoption<'open, Schema> {
    pub(in crate::domain_computation::primary_graph) predecessor: WorthQueryOpenAdoptionPredecessor,
    pub(in crate::domain_computation::primary_graph) resources: WorthQueryOpenAdoptionResources,
    pub(in crate::domain_computation::primary_graph) author: Box<
        dyn FnOnce(
                &mut WorthQueryOpenAdoptionWriter<'_, Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'open,
    >,
}

impl<'open, Schema> WorthQueryOpenAdoption<'open, Schema> {
    pub fn new(
        predecessor: WorthQueryOpenAdoptionPredecessor,
        resources: WorthQueryOpenAdoptionResources,
        author: impl FnOnce(
                &mut WorthQueryOpenAdoptionWriter<'_, Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'open,
    ) -> Self {
        Self {
            predecessor,
            resources,
            author: Box::new(author),
        }
    }

    /// Whether a recorded activation rendering names this adoption's predecessor.
    pub(in crate::domain_computation::primary_graph) fn adopts(
        &self,
        rendering: &worth_foundational::facade::AspectValue,
    ) -> bool {
        *rendering == self.predecessor.rendering()
    }
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
