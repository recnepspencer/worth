use sha2::Digest;
use worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart as Part;
use worth_ui_host_contract::UiCountedSha256;

use super::{
    WorthUiPresentationMechanicBasis, WorthUiPresentationPinBasis, WorthUiPresentationRequestBasis,
};

impl WorthUiPresentationRequestBasis {
    pub(crate) fn identity_parts(&self) -> Vec<Part> {
        vec![
            Part::unsigned("identity-schema", 4),
            Part::unsigned("mounted-frame", self.mounted_frame.diagnostic_value()),
            Part::unsigned("semantic-surface", self.semantic_surface.diagnostic_value()),
            Part::unsigned("host-surface", self.host_surface.diagnostic_value()),
            Part::unsigned("surface-binding", self.binding.diagnostic_value()),
            Part::unsigned("complete-frame", u64::from(self.complete)),
            Part::unsigned("dpi-milli", u64::from(self.dpi_milli)),
            Part::unsigned("presentation-attempt", self.attempt.diagnostic_value()),
            Part::unsigned("host-lineage", self.host_lineage.diagnostic_value()),
            Part::unsigned(
                "predecessor-frame",
                self.predecessor.map_or(0, |value| value.diagnostic_value()),
            ),
            Part::unsigned("mechanic-count", count(self.mechanics.len())),
            Part::bytes32(
                "mechanics-fingerprint",
                mechanics_fingerprint(&self.mechanics),
            ),
            Part::unsigned(
                "removed-mechanic-count",
                count(self.removed_mechanics.len()),
            ),
            Part::bytes32(
                "removed-mechanics-fingerprint",
                removed_mechanics_fingerprint(&self.removed_mechanics),
            ),
            Part::unsigned("binding-pin-count", count(self.binding_pins.len())),
            Part::bytes32(
                "binding-pins-fingerprint",
                pins_fingerprint(b"binding", &self.binding_pins),
            ),
            Part::unsigned("pin-addition-count", count(self.pin_additions.len())),
            Part::bytes32(
                "pin-additions-fingerprint",
                pins_fingerprint(b"addition", &self.pin_additions),
            ),
            Part::unsigned("pin-release-count", count(self.pin_releases.len())),
            Part::bytes32(
                "pin-releases-fingerprint",
                pins_fingerprint(b"release", &self.pin_releases),
            ),
        ]
    }

    #[doc(hidden)]
    pub fn active_mechanic_identity_digest(&self) -> [u8; 32] {
        mechanic_identity_digest(
            b"active",
            self.mechanics.iter().map(|mechanic| mechanic.mechanic),
        )
    }

    #[doc(hidden)]
    pub fn removed_mechanic_identity_digest(&self) -> [u8; 32] {
        mechanic_identity_digest(b"removed", self.removed_mechanics.iter().copied())
    }
}

fn mechanic_identity_digest(
    posture: &[u8],
    mechanics: impl Iterator<Item = worth_ui_host_contract::UiMountedPaintCommandIdentity>,
) -> [u8; 32] {
    let mechanics = mechanics.collect::<Vec<_>>();
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui/presentation-mechanic-identities/v1\0");
    encode_bytes(&mut digest, b"posture", posture);
    encode_count(&mut digest, mechanics.len());
    for mechanic in mechanics {
        encode_paint_command(&mut digest, mechanic);
    }
    digest.finalize().into()
}

fn mechanics_fingerprint(mechanics: &[WorthUiPresentationMechanicBasis]) -> [u8; 32] {
    let mut digest = fingerprint(b"mechanics");
    encode_count(&mut digest, mechanics.len());
    for mechanic in mechanics {
        encode_u64(
            &mut digest,
            b"mounted-instance",
            mechanic.mounted_instance.diagnostic_value(),
        );
        encode_paint_command(&mut digest, mechanic.mechanic);
        encode_u64(
            &mut digest,
            b"content-generation",
            mechanic.content_generation.diagnostic_value(),
        );
        encode_bytes(&mut digest, b"content", mechanic.content.as_bytes());
        encode_bytes(
            &mut digest,
            b"layout-request",
            &mechanic.layout_request.digest(),
        );
        encode_u64(
            &mut digest,
            b"layout-width",
            u64::from(mechanic.layout_width.width_millipoints()),
        );
        encode_bytes(&mut digest, b"layout", &mechanic.layout.digest());
        encode_count(&mut digest, mechanic.raster_key_set.keys().len());
        for key in mechanic.raster_key_set.keys() {
            encode_raster_key(&mut digest, *key);
        }
        encode_u64(&mut digest, b"text-scale", mechanic.text_scale.get());
        encode_count(&mut digest, mechanic.paint_spans.len());
        for span in &mechanic.paint_spans {
            encode_bytes(&mut digest, b"span-identity", &span.identity);
            encode_u32(&mut digest, b"span-start", span.original_range.start());
            encode_u32(&mut digest, b"span-end", span.original_range.end());
            encode_bytes(&mut digest, b"span-foreground", &span.foreground);
        }
    }
    digest.finalize().into()
}

fn removed_mechanics_fingerprint(
    mechanics: &[worth_ui_host_contract::UiMountedPaintCommandIdentity],
) -> [u8; 32] {
    let mut digest = fingerprint(b"removed-mechanics");
    encode_count(&mut digest, mechanics.len());
    for mechanic in mechanics {
        encode_paint_command(&mut digest, *mechanic);
    }
    digest.finalize().into()
}

fn encode_paint_command(
    digest: &mut UiCountedSha256,
    mechanic: worth_ui_host_contract::UiMountedPaintCommandIdentity,
) {
    let (slot, row) = mechanic
        .semantic_text_identity_parts()
        .unwrap_or((u16::MAX, None));
    encode_u64(
        digest,
        b"command-mounted-instance",
        mechanic.mounted_instance().diagnostic_value(),
    );
    encode_u64(digest, b"command-semantic-slot", u64::from(slot));
    encode_u64(digest, b"command-row-present", u64::from(row.is_some()));
    if let Some(row) = row {
        encode_bytes(digest, b"command-row", &row);
    }
}

pub(super) type PinSortKey = ([u8; 32], super::raster_key_set::RasterKeySortKey);

/// Pins order by layout, then by their raster key's sort key.
pub(super) fn pin_sort_key(pin: &WorthUiPresentationPinBasis) -> PinSortKey {
    (
        pin.layout().digest(),
        super::raster_key_set::key_sort_key(&pin.key()),
    )
}

fn pins_fingerprint(domain: &[u8], pins: &[WorthUiPresentationPinBasis]) -> [u8; 32] {
    let mut digest = fingerprint(domain);
    encode_count(&mut digest, pins.len());
    for pin in pins {
        encode_pin(&mut digest, *pin);
    }
    digest.finalize().into()
}

fn encode_pin(digest: &mut UiCountedSha256, pin: WorthUiPresentationPinBasis) {
    encode_bytes(digest, b"pin-layout", &pin.layout().digest());
    encode_raster_key(digest, pin.key());
}

/// A key enters the digest as its canonical evidence: every profile field in
/// fixed widths plus the axis count, so one length-prefixed field is injective.
fn encode_raster_key(digest: &mut UiCountedSha256, key: worth_ui_host_contract::UiGlyphRasterKey) {
    encode_bytes(digest, b"raster-key", &key.canonical_evidence_bytes());
}

fn fingerprint(domain: &[u8]) -> UiCountedSha256 {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui/presentation-request-identity/v2\0");
    encode_bytes(&mut digest, b"collection-domain", domain);
    digest
}

fn encode_count(digest: &mut UiCountedSha256, value: usize) {
    encode_u64(digest, b"count", count(value));
}

fn encode_u32(digest: &mut UiCountedSha256, field: &[u8], value: u32) {
    encode_bytes(digest, field, &value.to_le_bytes());
}

fn encode_u64(digest: &mut UiCountedSha256, field: &[u8], value: u64) {
    encode_bytes(digest, field, &value.to_le_bytes());
}

fn encode_bytes(digest: &mut UiCountedSha256, field: &[u8], value: &[u8]) {
    digest.update((field.len() as u64).to_le_bytes());
    digest.update(field);
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

fn count(value: usize) -> u64 {
    u64::try_from(value).expect("admitted presentation identity length fits u64")
}
