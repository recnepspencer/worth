//! Original native creation intent, independent of publication and read permission.
use std::marker::PhantomData;
use worth_relational::facade::identity::EntityId;

mod committed_observation;

/// Opaque creation-intent data issued by an authentic prepared workflow start.
///
/// Retain this before execution to locate the original instance in a later
/// governed read. It proves neither existence nor permission. Migration and fork
/// preparation issue their successor's intent without changing the source's.
///
/// The native getter returns owned data without input or schema Clone bounds.
/// ```
/// use worth_query_execution::publication_boundary::workflow_instance::{
///     PreparedWorkflowInstanceStart, WorthQueryWorkflowInstanceDiscoveryIntent,
/// };
/// fn retain<S, O, I, C>(prepared: PreparedWorkflowInstanceStart<S, O, I, C>)
///     -> WorthQueryWorkflowInstanceDiscoveryIntent<S>
/// {
///     let original = prepared.discovery_intent();
///     drop(prepared);
///     assert_eq!(original, original.clone());
///     original
/// }
/// ```
/// Construction is confined to the native canonical owner.
/// ```compile_fail,E0624
/// use worth_query_execution::publication_boundary::workflow_instance::WorthQueryWorkflowInstanceDiscoveryIntent;
/// fn forge<S>() -> WorthQueryWorkflowInstanceDiscoveryIntent<S> {
///     WorthQueryWorkflowInstanceDiscoveryIntent::from_canonical(todo!(), todo!(), todo!()).unwrap()
/// }
/// ```
/// Distinct schema identities cannot be substituted.
/// ```compile_fail,E0308
/// use worth_query_execution::publication_boundary::workflow_instance::WorthQueryWorkflowInstanceDiscoveryIntent;
/// fn substitute<A, B>(original: WorthQueryWorkflowInstanceDiscoveryIntent<A>)
///     -> WorthQueryWorkflowInstanceDiscoveryIntent<B>
/// { original }
/// ```
pub struct WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    identity_text: [u8; 64],
    intent_identity: [u8; 32],
    subject: EntityId,
    original_start_revision:
        worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    pub(super) fn from_canonical(
        subject: EntityId,
        original_start_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        (text, intent_identity): (String, [u8; 32]),
    ) -> Result<Self, ()> {
        let identity_text: [u8; 64] = text.as_bytes().try_into().map_err(|_| ())?;
        if !identity_text
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        {
            return Err(());
        }
        Ok(Self {
            identity_text,
            intent_identity,
            subject,
            original_start_revision,
            schema: PhantomData,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn lookup_identity(&self) -> &str {
        std::str::from_utf8(&self.identity_text).expect("native canonical identity is ASCII")
    }

    pub(in crate::domain_computation::primary_graph) const fn subject_entity_id(&self) -> EntityId {
        self.subject
    }

    /// Native producer metadata already bound into the unchanged canonical digest.
    pub(in crate::domain_computation::primary_graph) const fn original_start_revision(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramRevision {
        &self.original_start_revision
    }

    pub(super) const fn intent_identity(&self) -> &[u8; 32] {
        &self.intent_identity
    }
}

impl<Schema> Clone for WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    fn clone(&self) -> Self {
        Self {
            identity_text: self.identity_text,
            intent_identity: self.intent_identity,
            subject: self.subject,
            original_start_revision: self.original_start_revision,
            schema: PhantomData,
        }
    }
}

impl<Schema> PartialEq for WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.identity_text == other.identity_text
            && self.intent_identity == other.intent_identity
            && self.subject == other.subject
            && self.original_start_revision == other.original_start_revision
    }
}

impl<Schema> Eq for WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {}

impl<Schema> std::fmt::Debug for WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WorthQueryWorkflowInstanceDiscoveryIntent { .. }")
    }
}
