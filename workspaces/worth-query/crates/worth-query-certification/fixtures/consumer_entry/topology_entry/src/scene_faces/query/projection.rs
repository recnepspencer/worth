use super::*;
impl<
        S: SceneFaceSchemaBinding,
        Q: ApplicationQueryMarkerIdentity<S, ResultBinding = SceneFaceRowBinding>,
    > WorthQueryApplicationProjection<S, Q> for SceneFaceRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let face = row.one(face())?;
        let anchor_id = row.entity_id();
        let face_id = face.entity_id();
        Ok(Self {
            anchor: (
                anchor_id.partition_value(),
                anchor_id.local_slot_value(),
                anchor_id.generation_value(),
            ),
            face: (
                face_id.partition_value(),
                face_id.local_slot_value(),
                face_id.generation_value(),
            ),
            face_key: face.field(face_key())?,
            anchor_key: row.field(anchor_key())?,
            points: points(row)?,
        })
    }
}

/// Borrow just the physical coordinates; bounds do not need copied display keys.
pub(in crate::scene_faces) fn points<
    S: SceneFaceSchemaBinding,
    Q: ApplicationQueryMarkerIdentity<S, ResultBinding = SceneFaceRowBinding>,
>(
    row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
) -> Result<[[u64; 2]; 3], WorthQueryApplicationProjectionDenial> {
    let next = row.one(next())?;
    let previous = row.one(previous())?;
    Ok([
        [
            PositiveLength::get(&row.field(root_x())?),
            PositiveLength::get(&row.field(root_y())?),
        ],
        [
            PositiveLength::get(&next.field(next_x())?),
            PositiveLength::get(&next.field(next_y())?),
        ],
        [
            PositiveLength::get(&previous.field(previous_x())?),
            PositiveLength::get(&previous.field(previous_y())?),
        ],
    ])
}
