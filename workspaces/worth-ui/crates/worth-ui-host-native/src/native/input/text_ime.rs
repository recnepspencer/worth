use std::ops::Range;

use winit::event::Ime;
use worth_ui_host_contract::{
    UiHostImeCompositionPhase, UiHostImePreedit, UiHostImePreeditConstructionDenial,
    UiHostObservationPayload,
};

/// Whether an IME composition is in progress, as the composition owner
/// tracks it. `Ime::Disabled` cancels a composition only while one is in
/// progress; a window that never composed receives the same event as the
/// platform's availability notice (X11 sends it at window creation), and that
/// notice is not input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeImeCompositionPosture {
    Idle,
    Composing,
}

#[derive(Debug)]
pub(crate) enum UiNativeImeDenial {
    RevisionExhausted,
    RangeNotScalarBoundary,
    Preedit(UiHostImePreeditConstructionDenial),
}

pub(crate) fn translate(
    event: &Ime,
    next_revision: &mut Option<u64>,
    composition_active: &mut bool,
) -> Result<Option<UiHostObservationPayload>, UiNativeImeDenial> {
    let phase = match event {
        Ime::Enabled => {
            *composition_active = false;
            return Ok(None);
        }
        Ime::Disabled if !*composition_active => return Ok(None),
        Ime::Disabled => {
            *composition_active = false;
            UiHostImeCompositionPhase::Cancel
        }
        Ime::Preedit(text, _) if text.is_empty() => {
            *composition_active = false;
            UiHostImeCompositionPhase::Cancel
        }
        Ime::Preedit(text, active_range) => {
            let active_range = active_range
                .map(|(start, end)| scalar_range(text, start..end))
                .transpose()
                .map_err(|_| UiNativeImeDenial::RangeNotScalarBoundary)?;
            *composition_active = true;
            UiHostImeCompositionPhase::Preedit(
                UiHostImePreedit::from_unicode_scalar_range(text.clone(), active_range)
                    .map_err(UiNativeImeDenial::Preedit)?,
            )
        }
        Ime::Commit(text) => {
            *composition_active = false;
            UiHostImeCompositionPhase::Commit(text.clone().into_boxed_str())
        }
    };
    let revision = next_revision
        .take()
        .ok_or(UiNativeImeDenial::RevisionExhausted)?;
    *next_revision = revision.checked_add(1);
    Ok(Some(UiHostObservationPayload::ImeComposition {
        revision,
        phase,
    }))
}

fn scalar_range(text: &str, byte_range: Range<usize>) -> Result<Range<usize>, ()> {
    if byte_range.start > byte_range.end
        || byte_range.end > text.len()
        || !text.is_char_boundary(byte_range.start)
        || !text.is_char_boundary(byte_range.end)
    {
        return Err(());
    }
    Ok(text[..byte_range.start].chars().count()..text[..byte_range.end].chars().count())
}
