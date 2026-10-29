use super::UiDraftFieldIdentity;

pub const UI_DRAFT_SESSION_LIMIT: usize = 16;
pub const UI_DRAFT_UTF8_BYTE_LIMIT: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiDraftByteBudget(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDraftByteBudgetDenial {
    Empty,
    ExceedsRuntimeLimit { requested: usize, limit: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDraftRecipientContractDenial {
    InvalidPayloadSchema(crate::capability::UiIntentPayloadSchemaViolation),
    FieldOutsidePayloadSchema { slot: u8 },
    FieldHandleMismatch { slot: u8 },
    ByteBudget(UiDraftByteBudgetDenial),
}

pub use worth_ui_host_contract::UiHostInputRecipientFamily as UiLocalInputRecipientFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiLocalInputRecipientContract {
    kind: UiLocalInputRecipientContractKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiLocalInputRecipientContractKind {
    Activation,
    Draft {
        field: UiDraftFieldIdentity,
        budget: UiDraftByteBudget,
    },
    Submit,
}

impl UiDraftByteBudget {
    pub fn new(utf8_bytes: usize) -> Result<Self, UiDraftByteBudgetDenial> {
        if utf8_bytes == 0 {
            return Err(UiDraftByteBudgetDenial::Empty);
        }
        if utf8_bytes > UI_DRAFT_UTF8_BYTE_LIMIT {
            return Err(UiDraftByteBudgetDenial::ExceedsRuntimeLimit {
                requested: utf8_bytes,
                limit: UI_DRAFT_UTF8_BYTE_LIMIT,
            });
        }
        Ok(Self(
            u32::try_from(utf8_bytes).expect("the runtime draft limit fits u32"),
        ))
    }

    pub const fn utf8_bytes(self) -> usize {
        self.0 as usize
    }
}

impl UiLocalInputRecipientContract {
    pub const fn activation() -> Self {
        Self {
            kind: UiLocalInputRecipientContractKind::Activation,
        }
    }

    pub fn draft<P: crate::capability::UiIntentPayload>(
        field: crate::capability::UiIntentPayloadField<P, crate::capability::UiIntentText>,
    ) -> Result<Self, UiDraftRecipientContractDenial> {
        let descriptor = field.descriptor();
        let budget = draft_text_field(descriptor, P::FIELDS)?;
        Ok(Self {
            kind: UiLocalInputRecipientContractKind::Draft {
                field: UiDraftFieldIdentity::from_payload_field(field),
                budget,
            },
        })
    }

    pub const fn submit() -> Self {
        Self {
            kind: UiLocalInputRecipientContractKind::Submit,
        }
    }

    pub const fn family(self) -> UiLocalInputRecipientFamily {
        match self.kind {
            UiLocalInputRecipientContractKind::Activation => {
                UiLocalInputRecipientFamily::Activation
            }
            UiLocalInputRecipientContractKind::Draft { .. } => UiLocalInputRecipientFamily::Draft,
            UiLocalInputRecipientContractKind::Submit => UiLocalInputRecipientFamily::Submit,
        }
    }

    pub(super) const fn kind(self) -> UiLocalInputRecipientContractKind {
        self.kind
    }
}

#[inline(never)]
fn draft_text_field(
    descriptor: crate::capability::UiIntentPayloadFieldDescriptor,
    fields: crate::capability::UiIntentPayloadFieldSet,
) -> Result<UiDraftByteBudget, UiDraftRecipientContractDenial> {
    fields
        .validate()
        .map_err(UiDraftRecipientContractDenial::InvalidPayloadSchema)?;
    let declared = fields.fields().get(usize::from(descriptor.slot())).ok_or(
        UiDraftRecipientContractDenial::FieldOutsidePayloadSchema {
            slot: descriptor.slot(),
        },
    )?;
    if declared != &descriptor {
        return Err(UiDraftRecipientContractDenial::FieldHandleMismatch {
            slot: descriptor.slot(),
        });
    }
    UiDraftByteBudget::new(descriptor.byte_budget())
        .map_err(UiDraftRecipientContractDenial::ByteBudget)
}
