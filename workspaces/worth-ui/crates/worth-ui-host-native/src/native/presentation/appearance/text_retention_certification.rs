#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeTextRetentionCertificationDenial {
    Scale,
    Insert,
    Replace,
    Identity,
    Gap,
    Remove,
}

/// Exercise the existing staged retained owner using only genuinely finalized
/// foreground. This proves candidate retention, not presentation acceptance.
pub(crate) fn certify_retention(
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    first: super::text_foreground::UiNativeFinalizedTextForeground,
    second: super::text_foreground::UiNativeFinalizedTextForeground,
    successor: super::text_foreground::UiNativeFinalizedTextForeground,
    gap: [i64; 4],
) -> Result<Box<[[i64; 4]]>, UiNativeTextRetentionCertificationDenial> {
    use crate::native::presentation::appearance::{
        command::UiNativeAppearanceCommand as Command, damage::UiNativeAppearanceDamageRect,
        geometry::UiNativeAppearanceScale, retained::UiNativeAppearanceRetained,
    };
    let scale = UiNativeAppearanceScale::qualified(1_000)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Scale)?;
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let (_, inserted) = retained
        .stage_text_insert(successor.clone(), atlas, None)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Insert)?;
    retained
        .rollback_text(inserted)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Insert)?;
    if !retained.ordered_keys().is_empty() || !retained.take_damage().is_empty() {
        return Err(UiNativeTextRetentionCertificationDenial::Insert);
    }
    // Historical coverage is seeded from real finalized payloads. It does not
    // claim that this certification function performed their presentation.
    let key = retained
        .insert(Command::TextForeground(first), None)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Insert)?;
    let second_command = Command::TextForeground(second);
    let other = retained
        .insert(second_command.clone(), Some(key))
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Insert)?;
    retained.take_damage();
    let previous = retained.command(key).cloned();
    let Some(Command::TextForeground(stale)) = previous.clone() else {
        return Err(UiNativeTextRetentionCertificationDenial::Identity);
    };
    if !matches!(
        retained.stage_text_replace(stale, atlas),
        Err(super::retained::UiNativeAppearanceRetainedDenial::StaleTextImages)
    ) || retained.command(key) != previous.as_ref()
        || !retained.take_damage().is_empty()
    {
        return Err(UiNativeTextRetentionCertificationDenial::Replace);
    }
    let replaced = retained
        .stage_text_replace(successor.clone(), atlas)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Replace)?;
    retained
        .rollback_text(replaced)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Replace)?;
    if retained.command(key) != previous.as_ref()
        || retained.command(other) != Some(&second_command)
        || !retained.take_damage().is_empty()
    {
        return Err(UiNativeTextRetentionCertificationDenial::Replace);
    }
    let removed = retained
        .stage_text_remove(key)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Remove)?;
    retained
        .rollback_text(removed)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Remove)?;
    if retained.command(key) != previous.as_ref()
        || retained.ordered_keys().as_ref() != [key, other]
        || !retained.take_damage().is_empty()
    {
        return Err(UiNativeTextRetentionCertificationDenial::Remove);
    }
    let Some(Command::TextForeground(restored)) = retained.command(key) else {
        return Err(UiNativeTextRetentionCertificationDenial::Identity);
    };
    let Some(mut old_edge) = restored.coverage().first().copied() else {
        return Err(UiNativeTextRetentionCertificationDenial::Gap);
    };
    old_edge.right = old_edge.left + 1;
    if retained
        .replay_for_damage(old_edge)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Gap)?
        .as_ref()
        != [key]
    {
        return Err(UiNativeTextRetentionCertificationDenial::Gap);
    }
    let successor = Command::TextForeground(successor);
    if retained
        .replace(successor.clone())
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Replace)?
        != key
    {
        return Err(UiNativeTextRetentionCertificationDenial::Identity);
    }
    if retained.command(key) != Some(&successor) || retained.command(other) != Some(&second_command)
    {
        return Err(UiNativeTextRetentionCertificationDenial::Identity);
    }
    let damage = retained.take_damage();
    let replay = retained
        .replay_for_damage(UiNativeAppearanceDamageRect {
            left: gap[0],
            top: gap[1],
            right: gap[2],
            bottom: gap[3],
        })
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Gap)?;
    if !replay.is_empty() {
        return Err(UiNativeTextRetentionCertificationDenial::Gap);
    }
    retained
        .remove(key)
        .map_err(|_| UiNativeTextRetentionCertificationDenial::Remove)?;
    if retained.ordered_keys().as_ref() != [other]
        || retained.command(other) != Some(&second_command)
    {
        return Err(UiNativeTextRetentionCertificationDenial::Identity);
    }
    Ok(damage
        .iter()
        .map(|r| [r.left, r.top, r.right, r.bottom])
        .collect())
}
