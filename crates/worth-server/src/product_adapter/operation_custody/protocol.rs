//! Create opens typed command custody; controls cannot replace its original input.
use self::{WorthServerSourceCommand as Command, WorthServerSourceReply as Reply};
use super::contract::WorthServerQuerySourceAdapter as SourceWire;
use crate::{
    WorthServerProductOperationDenial, WorthServerProductOperationPayload,
    WorthServerProductPayloadSchemaValidator, WorthServerProductResultContract,
    WorthServerProductResultValue,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(tag = "command", deny_unknown_fields)]
pub enum WorthServerSourceCommand<I> {
    Create {
        custody_id: String,
        logical_key: String,
        intent: I,
    },
    Advance {
        custody_id: String,
        sequence: u64,
    },
    Inspect {
        custody_id: String,
    },
    Close {
        custody_id: String,
        sequence: u64,
    },
}
impl<I> WorthServerSourceCommand<I> {
    pub(super) fn custody_id(&self) -> &str {
        match self {
            Self::Create { custody_id, .. }
            | Self::Advance { custody_id, .. }
            | Self::Inspect { custody_id }
            | Self::Close { custody_id, .. } => custody_id,
        }
    }
}
pub(super) struct CommandInput<C>(std::marker::PhantomData<fn() -> C>);
impl<C> Default for CommandInput<C> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<C: SourceWire> WorthServerProductPayloadSchemaValidator for CommandInput<C> {
    fn validate(
        &self,
        payload: &WorthServerProductOperationPayload,
    ) -> Result<(), WorthServerProductOperationDenial> {
        decode::<C>(payload).map(|_| ())
    }
}
pub(super) fn decode<C: SourceWire>(
    payload: &WorthServerProductOperationPayload,
) -> Result<Command<C::Input>, WorthServerProductOperationDenial> {
    if payload.envelope().declared_schema_identity() != Some(C::INPUT) {
        return Err(denied::<C>("input_identity"));
    }
    let command: Command<C::Input> =
        serde_json::from_value(payload.body().clone()).map_err(|_| denied::<C>("input"))?;
    if !valid_key(command.custody_id()) {
        return Err(denied::<C>("custody_identity"));
    }
    if let Command::Create { logical_key, .. } = &command
        && !valid_key(logical_key)
    {
        return Err(denied::<C>("logical_key"));
    }
    Ok(command)
}
fn valid_key(key: &str) -> bool {
    !key.trim().is_empty() && key.len() <= 128
}
fn denied<C: SourceWire>(reason: &str) -> WorthServerProductOperationDenial {
    WorthServerProductOperationDenial::new(
        format!("{}.{reason}", C::NAMESPACE),
        "invalid native source control",
    )
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(tag = "kind")]
pub enum WorthServerSourceReply {
    Held {
        custody_id: String,
        next_sequence: Option<u64>,
        native: super::view::View,
    },
    Closed {
        custody_id: String,
    },
    PreparationDiscarded {
        custody_id: String,
    },
    NotExecuted {
        custody_id: String,
        reason: ControlRefusal,
    },
    OutcomeUnknown {
        custody_id: String,
    },
    DisclosureRefused {
        custody_id: String,
        diagnostic: Option<super::diagnostic::WorthServerSourceDiagnostic>,
    },
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum ControlRefusal {
    CustodyUnavailable,
    AlreadyHeld,
    LogicalKeyHeld,
    Capacity,
    StaleSequence,
    Quiesced,
    QueueFull,
    TransportUnavailable,
}
#[derive(Serialize, schemars::JsonSchema)]
pub(super) struct PublishedReply<'a> {
    #[serde(flatten)]
    pub(super) value: &'a Reply,
    #[serde(skip)]
    pub(super) identity: &'static str,
}
impl WorthServerProductResultValue for PublishedReply<'_> {
    fn result_schema_identity(&self) -> &str {
        self.identity
    }
    fn result_schema_version(&self) -> u32 {
        3
    }
}
pub(super) fn result_contract(identity: &str) -> WorthServerProductResultContract {
    WorthServerProductResultContract::canonical_json(identity, 3, 16 * 1024)
        .expect("fixed owning source result")
}
