const ROLE_CANONICAL_VERSION: &[u8] = b"worth-ui:appearance-role:v2";

pub(super) fn role_bytes(role: &super::UiAppearanceRoleDeclaration) -> Vec<u8> {
    let mut bytes = ROLE_CANONICAL_VERSION.to_vec();
    text(&mut bytes, role.role().as_str());
    bytes.extend_from_slice(&role.schema().revision().to_le_bytes());
    bytes.extend_from_slice(&role.revision().value().to_le_bytes());
    match role.applicability() {
        super::UiAppearanceRoleApplicability::AnyComponent => bytes.push(1),
        super::UiAppearanceRoleApplicability::Component(component) => {
            bytes.push(2);
            text(&mut bytes, component.as_str());
        }
        super::UiAppearanceRoleApplicability::Backdrop => bytes.push(3),
    }
    bytes.push(aspect_applicability_tag(
        role.aspect_contract().applicability(),
    ));
    aspects(&mut bytes, role.aspect_contract().required());
    aspects(&mut bytes, role.aspect_contract().optional());
    put_len(&mut bytes, role.partitions().len());
    for (aspect, partition) in role.partitions() {
        bytes.push(aspect_tag(*aspect));
        put_len(&mut bytes, partition.axes().len());
        for axis in partition.axes() {
            bytes.push(axis_tag(axis.axis()));
            bytes.extend_from_slice(&axis.revision().to_le_bytes());
        }
        put_len(&mut bytes, partition.cells().len());
        for cell in partition.cells() {
            put_len(&mut bytes, cell.classes().len());
            for class in cell.classes() {
                bytes.push(class_tag(*class));
            }
            result(&mut bytes, cell.result());
        }
    }
    bytes
}

fn aspects(bytes: &mut Vec<u8>, values: &[super::UiAppearanceAspect]) {
    put_len(bytes, values.len());
    for aspect in values {
        bytes.push(aspect_tag(*aspect));
    }
}

fn result(bytes: &mut Vec<u8>, result: &super::UiAppearanceDecisionResult) {
    bytes.push(value_kind_tag(result.value_kind()));
    match result.value() {
        super::UiAppearanceDecisionValue::ThemeSlot(slot) => {
            bytes.push(1);
            text(bytes, slot.as_str());
        }
        super::UiAppearanceDecisionValue::Literal(value) => {
            bytes.push(2);
            literal(bytes, *value);
        }
    }
}

fn literal(bytes: &mut Vec<u8>, value: super::UiThemeValue) {
    match value {
        super::UiThemeValue::Color(color) => {
            bytes.push(1);
            bytes.extend_from_slice(&color.channels());
        }
        super::UiThemeValue::Opacity(opacity) => {
            bytes.push(2);
            bytes.extend_from_slice(&opacity.units().to_le_bytes());
        }
        super::UiThemeValue::LogicalLength(length) => {
            bytes.push(3);
            bytes.extend_from_slice(&length.subpixels().to_le_bytes());
        }
        super::UiThemeValue::CornerRadii(radii) => {
            bytes.push(4);
            for length in radii.corners() {
                bytes.extend_from_slice(&length.subpixels().to_le_bytes());
            }
        }
        super::UiThemeValue::SolidStroke(stroke) => {
            bytes.push(5);
            bytes.extend_from_slice(&stroke.color().channels());
            bytes.extend_from_slice(&stroke.width().subpixels().to_le_bytes());
        }
        super::UiThemeValue::SolidOutline(outline) => {
            bytes.push(6);
            let stroke = outline.stroke();
            bytes.extend_from_slice(&stroke.color().channels());
            bytes.extend_from_slice(&stroke.width().subpixels().to_le_bytes());
            bytes.extend_from_slice(&outline.offset().subpixels().to_le_bytes());
        }
    }
}

fn aspect_tag(aspect: super::UiAppearanceAspect) -> u8 {
    match aspect {
        super::UiAppearanceAspect::Background => 1,
        super::UiAppearanceAspect::Foreground => 2,
        super::UiAppearanceAspect::Border => 3,
        super::UiAppearanceAspect::Radius => 4,
        super::UiAppearanceAspect::Opacity => 5,
        super::UiAppearanceAspect::Outline => 6,
    }
}

fn aspect_applicability_tag(applicability: super::UiAppearanceAspectApplicability) -> u8 {
    match applicability {
        super::UiAppearanceAspectApplicability::Component => 1,
        super::UiAppearanceAspectApplicability::Backdrop => 2,
    }
}

fn axis_tag(axis: super::UiAppearanceStateAxis) -> u8 {
    match axis {
        super::UiAppearanceStateAxis::Operability => 1,
        super::UiAppearanceStateAxis::Focus => 2,
        super::UiAppearanceStateAxis::Validation => 3,
        super::UiAppearanceStateAxis::Selection => 4,
        super::UiAppearanceStateAxis::Hover => 5,
        super::UiAppearanceStateAxis::Pressed => 6,
    }
}

fn class_tag(class: super::UiAppearanceAxisClass) -> u8 {
    use super::UiAppearanceAxisClass::*;
    match class {
        OperabilityReady => 1,
        OperabilityPending => 2,
        OperabilityOccupied => 3,
        OperabilityDenied => 4,
        OperabilityUnsupported => 5,
        OperabilityStale => 6,
        FocusUnfocused => 7,
        FocusFocused => 8,
        FocusVisible => 9,
        FocusedWindowInactive => 10,
        ValidationUnspecified => 11,
        ValidationValid => 12,
        ValidationAdvisory => 13,
        ValidationInvalid => 14,
        ValidationPending => 15,
        ValidationStale => 16,
        SelectionUnselected => 17,
        SelectionSelected => 18,
        SelectionAnchor => 19,
        SelectionCursor => 20,
        SelectedAnchorCursor => 21,
        HoverOutside => 22,
        Hovered => 23,
        PressedIdle => 24,
        PressedArmedInside => 25,
        PressedCapturedOutside => 26,
    }
}

fn value_kind_tag(kind: super::UiThemeValueKind) -> u8 {
    match kind {
        super::UiThemeValueKind::Color => 1,
        super::UiThemeValueKind::Opacity => 2,
        super::UiThemeValueKind::LogicalLength => 3,
        super::UiThemeValueKind::CornerRadii => 4,
        super::UiThemeValueKind::SolidStroke => 5,
        super::UiThemeValueKind::SolidOutline => 6,
    }
}

fn put_len(bytes: &mut Vec<u8>, length: usize) {
    bytes.extend_from_slice(&(length as u64).to_le_bytes());
}

fn text(bytes: &mut Vec<u8>, value: &str) {
    put_len(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}
