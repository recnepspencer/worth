use worth_ui_host_contract::{UiFontSlant, UiQualifiedFontFaceIdentity};

use super::{face::UiQualifiedFontFace, UiGlobalFontCollection};

impl UiGlobalFontCollection {
    pub(crate) fn contains_family(
        &self,
        family: worth_ui_host_contract::UiQualifiedFontFamilyIdentity,
    ) -> bool {
        self.faces.iter().any(|face| face.family() == family)
    }

    pub(crate) fn face_identity(&self, slot: usize) -> UiQualifiedFontFaceIdentity {
        self.faces[slot].identity()
    }

    pub(crate) fn is_last_resort(&self, slot: usize) -> bool {
        self.faces[slot].is_last_resort()
    }

    pub(crate) fn contains_cluster(&self, slot: usize, text: &str, rgi_emoji: bool) -> bool {
        rgi_emoji || self.faces[slot].contains_cluster(text)
    }

    pub(crate) fn fallback_slots(
        &self,
        rgi_emoji: bool,
        style: &crate::UiTextStyle,
    ) -> impl Iterator<Item = usize> + use<> {
        let emoji = self.faces.iter().position(UiQualifiedFontFace::is_emoji);
        let mut slots = Vec::with_capacity(self.faces.len());
        let mut selected = vec![false; self.faces.len()];
        for family in style.family_stack().families() {
            let matching = self
                .faces
                .iter()
                .enumerate()
                .filter(|(_, face)| {
                    face.family() == *family && (!rgi_emoji || face.has_intrinsic_color())
                })
                .min_by_key(|(_, face)| face_match_key(face, style.face_request()));
            if let Some((slot, _)) = matching {
                push_once(&mut slots, &mut selected, slot);
            }
        }
        if rgi_emoji {
            if let Some(slot) = emoji {
                push_once(&mut slots, &mut selected, slot);
            }
        }
        if !rgi_emoji {
            for (slot, _) in self.faces.iter().enumerate().filter(|(_, face)| {
                face.pack().is_none() && !face.is_emoji() && !face.is_last_resort()
            }) {
                push_once(&mut slots, &mut selected, slot);
            }
        }
        if let Some(slot) = self
            .faces
            .iter()
            .position(UiQualifiedFontFace::is_last_resort)
        {
            push_once(&mut slots, &mut selected, slot);
        }
        slots.into_iter()
    }
}

fn push_once(slots: &mut Vec<usize>, selected: &mut [bool], slot: usize) {
    if !selected[slot] {
        slots.push(slot);
        selected[slot] = true;
    }
}

fn face_match_key(
    face: &UiQualifiedFontFace,
    request: crate::UiTextFaceRequest,
) -> (u8, u32, u32, u32, u16, u8, [u8; 32]) {
    let slant_penalty = if face.slant_supports(request.slant()) {
        0
    } else {
        slant_distance(face.slant(), request.slant())
    };
    (
        slant_penalty,
        face.width_distance(request.width_milli_percent()),
        face.weight_distance(request.weight()),
        face.width_milli_percent(),
        face.weight(),
        match face.slant() {
            UiFontSlant::Upright => 0,
            UiFontSlant::Italic => 1,
            UiFontSlant::Oblique => 2,
        },
        face.identity().selection_digest(),
    )
}

const fn slant_distance(face: UiFontSlant, request: UiFontSlant) -> u8 {
    match (request, face) {
        (UiFontSlant::Upright, UiFontSlant::Upright)
        | (UiFontSlant::Italic, UiFontSlant::Italic)
        | (UiFontSlant::Oblique, UiFontSlant::Oblique) => 0,
        (UiFontSlant::Italic, UiFontSlant::Oblique)
        | (UiFontSlant::Oblique, UiFontSlant::Italic) => 1,
        _ => 2,
    }
}
