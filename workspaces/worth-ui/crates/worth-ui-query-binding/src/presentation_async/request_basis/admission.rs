use std::collections::HashSet;

use super::*;

impl WorthUiPresentationRequestBasis {
    #[doc(hidden)]
    pub fn from_runtime_correspondence(
        input: WorthUiPresentationRequestBasisInput,
    ) -> Result<Self, WorthUiPresentationRequestBasisDenial> {
        if input.dpi_milli == 0 {
            return Err(WorthUiPresentationRequestBasisDenial::ZeroDpi);
        }
        let mut mechanics = input
            .mechanics
            .into_vec()
            .into_iter()
            .map(admit_mechanic)
            .collect::<Result<Vec<_>, _>>()?;
        mechanics.sort_by_key(mechanic_sort_key);
        if mechanics
            .windows(2)
            .any(|pair| pair[0].mechanic == pair[1].mechanic)
        {
            return Err(WorthUiPresentationRequestBasisDenial::DuplicateMechanic);
        }
        let mut removed_mechanics = input.removed_mechanics.into_vec();
        removed_mechanics.sort_by_key(paint_command_sort_key);
        if removed_mechanics.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WorthUiPresentationRequestBasisDenial::DuplicateRemovedMechanic);
        }
        if mechanics
            .iter()
            .any(|mechanic| removed_mechanics.contains(&mechanic.mechanic))
        {
            return Err(WorthUiPresentationRequestBasisDenial::MechanicAlsoRemoved);
        }
        let mut pin_additions = input.pin_additions.into_vec();
        let mut pin_releases = input.pin_releases.into_vec();
        let mut binding_pins = input.binding_pins.into_vec();
        sort_pins(&mut pin_additions);
        sort_pins(&mut pin_releases);
        sort_pins(&mut binding_pins);
        validate_unique_pins(
            &pin_additions,
            WorthUiPresentationRequestBasisDenial::DuplicatePinAddition,
        )?;
        validate_unique_pins(
            &pin_releases,
            WorthUiPresentationRequestBasisDenial::DuplicatePinRelease,
        )?;
        validate_unique_pins(
            &binding_pins,
            WorthUiPresentationRequestBasisDenial::DuplicateBindingPin,
        )?;
        Ok(Self {
            mounted_frame: input.mounted_frame,
            semantic_surface: input.semantic_surface,
            host_surface: input.host_surface,
            binding: input.binding,
            complete: input.complete,
            mechanics: mechanics.into_boxed_slice(),
            removed_mechanics: removed_mechanics.into_boxed_slice(),
            binding_pins: binding_pins.into_boxed_slice(),
            pin_additions: pin_additions.into_boxed_slice(),
            pin_releases: pin_releases.into_boxed_slice(),
            dpi_milli: input.dpi_milli,
            attempt: input.attempt,
            predecessor: input.predecessor,
            host_lineage: input.host_lineage,
        })
    }
}

fn admit_mechanic(
    input: WorthUiPresentationMechanicBasisInput,
) -> Result<WorthUiPresentationMechanicBasis, WorthUiPresentationRequestBasisDenial> {
    input
        .mechanic
        .semantic_text_identity_parts()
        .ok_or(WorthUiPresentationRequestBasisDenial::NonTextMechanic)?;
    if input.mechanic.mounted_instance() != input.mounted_instance {
        return Err(WorthUiPresentationRequestBasisDenial::MechanicMountedInstanceMismatch);
    }
    let mut paint_spans = input.paint_spans.into_vec();
    paint_spans.sort_by_key(|span| {
        (
            span.original_range.start(),
            span.original_range.end(),
            span.identity,
        )
    });
    validate_paint_spans(&paint_spans)?;
    Ok(WorthUiPresentationMechanicBasis {
        mounted_instance: input.mounted_instance,
        mechanic: input.mechanic,
        content_generation: input.content_generation,
        content: input.content,
        layout: input.layout,
        layout_request: input.layout_request,
        layout_width: input.layout_width,
        paint_spans: paint_spans.into_boxed_slice(),
        raster_key_set: WorthUiPresentationRasterKeySetBasis::from_runtime(
            input.raster_keys.into_vec(),
        ),
        text_scale: input.text_scale,
    })
}

fn validate_paint_spans(
    spans: &[WorthUiPresentationPaintSpanBasis],
) -> Result<(), WorthUiPresentationRequestBasisDenial> {
    let mut identities = HashSet::with_capacity(spans.len());
    let mut prior_end = None;
    for span in spans {
        if span.original_range.is_empty() {
            return Err(WorthUiPresentationRequestBasisDenial::EmptyPaintSpan);
        }
        if !identities.insert(span.identity) {
            return Err(WorthUiPresentationRequestBasisDenial::DuplicatePaintSpan);
        }
        if prior_end.is_some_and(|end| end > span.original_range.start()) {
            return Err(WorthUiPresentationRequestBasisDenial::OverlappingPaintSpan);
        }
        prior_end = Some(span.original_range.end());
    }
    Ok(())
}

fn mechanic_sort_key(mechanic: &WorthUiPresentationMechanicBasis) -> (u64, u32, Option<[u8; 32]>) {
    let (slot, row) = mechanic
        .mechanic
        .semantic_text_identity_parts()
        .expect("admitted presentation mechanic remains semantic text");
    (
        mechanic.mounted_instance.diagnostic_value(),
        u32::from(slot),
        row,
    )
}

fn paint_command_sort_key(
    mechanic: &UiMountedPaintCommandIdentity,
) -> (u64, u32, Option<[u8; 32]>) {
    let (slot, row) = mechanic
        .semantic_text_identity_parts()
        .unwrap_or((u16::MAX, None));
    (
        mechanic.mounted_instance().diagnostic_value(),
        u32::from(slot),
        row,
    )
}

fn sort_pins(pins: &mut [WorthUiPresentationPinBasis]) {
    pins.sort_by_key(super::identity_parts::pin_sort_parts);
}

fn validate_unique_pins(
    pins: &[WorthUiPresentationPinBasis],
    denial: WorthUiPresentationRequestBasisDenial,
) -> Result<(), WorthUiPresentationRequestBasisDenial> {
    let mut seen = HashSet::with_capacity(pins.len());
    for pin in pins {
        if !seen.insert(pin.pin) {
            return Err(denial);
        }
    }
    Ok(())
}
