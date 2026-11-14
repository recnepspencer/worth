//! One real typed envelope key and canonical identities of its authored stages.
use super::*;
use serde::Serialize;

/// A framework stage identity describes custody and result position. It is not
/// a domain-typed idempotency key or permission to perform a stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationEditStageIdentity {
    ordinal: usize,
    identity: [u8; 32],
}
impl ApplicationEditStageIdentity {
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }
}

/// The envelope owns the only caller key. Every stage borrows that same value,
/// with no conversion into an invented per-stage domain key (D1620).
pub struct ApplicationEditEnvelopeKey<Key> {
    key: Key,
    namespace: String,
    canonical: super::super::ApplicationCanonicalIdentity,
}
impl<Key: Serialize> ApplicationEditEnvelopeKey<Key> {
    pub fn encode(namespace: &str, key: Key) -> Result<Self, ApplicationValueEncodeDenial> {
        let canonical =
            super::super::canonical_identity("worth-query.edit-envelope-key.v1", namespace, &key)
                .map_err(|_| {
                super::super::rejected(
                    crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                        "worth-query.edit-envelope",
                    ),
                )
            })?;
        Ok(Self {
            key,
            namespace: namespace.to_owned(),
            canonical,
        })
    }

    pub fn key(&self) -> &Key {
        &self.key
    }
    pub const fn identity(&self) -> &[u8; 32] {
        &self.canonical.identity
    }
    pub const fn canonical_work(&self) -> ApplicationCanonicalWork {
        self.canonical.work
    }

    /// Each binding retains the envelope's real key type and value. Only its
    /// actual typed input is encoded here; the envelope key was encoded once.
    pub fn stage<'a, S, B>(
        &'a self,
        input: &'a B::Input,
        ordinal: usize,
    ) -> Result<ApplicationMutationIdentities<'a, S, B>, ApplicationMutationIdentityDenial>
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S, IdempotencyKey = Key>,
    {
        let encoded = input_identity::<B::InputBinding>(input)
            .map_err(ApplicationMutationIdentityDenial::Input)?;
        let stage = super::super::canonical_identity(
            "worth-query.edit-stage.v1",
            &self.namespace,
            &(self.canonical.identity, ordinal),
        )
        .map_err(|_| {
            ApplicationMutationIdentityDenial::Key(super::super::rejected(
                crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                    "worth-query.edit-envelope",
                ),
            ))
        })?;
        Ok(ApplicationMutationIdentities {
            key: &self.key,
            input: MutationInput::Borrowed(input),
            key_identity: self.canonical.identity,
            stage: Some(ApplicationEditStageIdentity {
                ordinal,
                identity: stage.identity,
            }),
            input_identity: encoded.canonical.identity,
            work: encoded.canonical.work.combine(stage.work),
            capture: encoded.capture,
            marker: PhantomData,
        })
    }
}

/// Canonical intent of the complete ordered stage list. The input and root
/// names come from the same nominal bindings that execute each stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ApplicationEditEnvelopeOrder {
    DeclaredDependencies,
    OrderedAuthored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationEditIntentIdentity {
    identity: [u8; 32],
    order: ApplicationEditEnvelopeOrder,
}
impl ApplicationEditIntentIdentity {
    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }
    pub const fn order(&self) -> ApplicationEditEnvelopeOrder {
        self.order
    }
}

/// Descriptive stage intent, with construction tied to an actual typed input.
#[derive(Serialize)]
pub struct ApplicationEditStageIntent {
    binding: &'static str,
    input: [u8; 32],
    root: Vec<EditConnectionIntent>,
}
#[derive(Serialize)]
struct EditConnectionIntent {
    identity: &'static str,
    source_instance: &'static str,
    source_feature: &'static str,
    source_port: &'static str,
    target_instance: &'static str,
    target_feature: &'static str,
    target_port: &'static str,
    required: bool,
    exports: bool,
    demand_only: bool,
}
impl ApplicationEditStageIntent {
    pub fn new<S, B, R>(input: &ApplicationEncodedInput<B::InputBinding>) -> Self
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        R: crate::application_program::ApplicationOutputGraphShape<S>,
    {
        let root = R::connections()
            .into_iter()
            .map(|connection| EditConnectionIntent {
                identity: connection.identity(),
                source_instance: connection.source_instance(),
                source_feature: connection.source_feature(),
                source_port: connection.source_port(),
                target_instance: connection.target_instance(),
                target_feature: connection.target_feature(),
                target_port: connection.target_port(),
                required: connection.target_required(),
                exports: connection.exports_across_instances(),
                demand_only: connection.is_demand_only_root(),
            })
            .collect();
        Self {
            binding: B::IDENTITY,
            input: *input.identity(),
            root,
        }
    }
}
impl<Key: Serialize> ApplicationEditEnvelopeKey<Key> {
    /// One full intent binds the explicit execution mode and original stage order.
    pub fn intent(
        &self,
        order: ApplicationEditEnvelopeOrder,
        stages: &[ApplicationEditStageIntent],
    ) -> Result<ApplicationEditIntentIdentity, ApplicationValueEncodeDenial> {
        super::super::canonical_identity(
            "worth-query.edit-envelope-intent.v1",
            &self.namespace,
            &(self.canonical.identity, order, stages),
        )
        .map(|canonical| ApplicationEditIntentIdentity {
            identity: canonical.identity,
            order,
        })
        .map_err(|_| {
            super::super::rejected(
                crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                    "worth-query.edit-envelope",
                ),
            )
        })
    }

    /// Retries borrow the original encoded input; they do not re-encode it.
    pub fn stage_with_encoded_input<'a, S, B>(
        &'a self,
        input: &ApplicationEncodedInput<B::InputBinding>,
        ordinal: usize,
    ) -> Result<ApplicationMutationIdentities<'a, S, B>, ApplicationMutationIdentityDenial>
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S, IdempotencyKey = Key>,
    {
        let stage = super::super::canonical_identity(
            "worth-query.edit-stage.v1",
            &self.namespace,
            &(self.canonical.identity, ordinal),
        )
        .map_err(|_| {
            ApplicationMutationIdentityDenial::Key(super::super::rejected(
                crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                    "worth-query.edit-envelope",
                ),
            ))
        })?;
        Ok(ApplicationMutationIdentities {
            key: &self.key,
            input: MutationInput::Shared(input.shared_input().clone()),
            key_identity: self.canonical.identity,
            stage: Some(ApplicationEditStageIdentity {
                ordinal,
                identity: stage.identity,
            }),
            input_identity: *input.identity(),
            work: input.canonical_work().combine(stage.work),
            capture: input.canonical_input().cloned(),
            marker: PhantomData,
        })
    }
}
