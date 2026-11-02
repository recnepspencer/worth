use std::{any::Any, sync::Arc};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};
use worth_relational::facade::content::*;

#[derive(Debug)]
pub(crate) struct Grid {
    pub rows: u32,
    pub columns: u32,
    pub samples: Vec<i64>,
}

#[derive(Debug)]
pub(crate) struct Codec {
    pub descriptor: ContentFormat,
    pub alter: bool,
}
impl RelationalContentCodec for Codec {
    fn format(&self) -> &ContentFormat {
        &self.descriptor
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        control.checkpoint(1)?;
        let grid = input
            .downcast_ref::<Grid>()
            .ok_or(RelationalContentDenial::InputTypeMismatch)?;
        if grid.rows == 0
            || grid.columns == 0
            || u64::from(grid.rows) * u64::from(grid.columns) != grid.samples.len() as u64
        {
            return Err(RelationalContentDenial::NonCanonicalContent);
        }
        grid.samples
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(8))
            .ok_or(RelationalContentDenial::ArithmeticOverflow)
    }
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        let grid = input
            .downcast_ref::<Grid>()
            .ok_or(RelationalContentDenial::InputTypeMismatch)?;
        sink.append(&grid.rows.to_le_bytes(), control)?;
        sink.append(&grid.columns.to_le_bytes(), control)?;
        for value in &grid.samples {
            sink.append(
                &value.wrapping_add(i64::from(self.alter)).to_le_bytes(),
                control,
            )?;
        }
        Ok(())
    }
    fn validate(
        &self,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        control.checkpoint(payload.len() as u64)?;
        let dimensions = payload
            .get(..8)
            .ok_or(RelationalContentDenial::NonCanonicalContent)?;
        let rows = u32::from_le_bytes(dimensions[..4].try_into().unwrap());
        let columns = u32::from_le_bytes(dimensions[4..].try_into().unwrap());
        if rows == 0
            || columns == 0
            || u128::from(rows) * u128::from(columns) * 8 + 8 != payload.len() as u128
        {
            return Err(RelationalContentDenial::NonCanonicalContent);
        }
        Ok(())
    }
    fn verify_encoding(
        &self,
        input: &dyn Any,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        let grid = input
            .downcast_ref::<Grid>()
            .ok_or(RelationalContentDenial::InputTypeMismatch)?;
        // Independent correspondence: inspect positions and signed integers;
        // never invoke encode or validate to prove what encode wrote.
        for (index, value) in grid.samples.iter().enumerate() {
            control.checkpoint(8)?;
            let word = payload
                .get(8 + index * 8..16 + index * 8)
                .ok_or(RelationalContentDenial::EncodingInconsistency)?;
            if i64::from_le_bytes(word.try_into().unwrap()) != *value {
                return Err(RelationalContentDenial::EncodingInconsistency);
            }
        }
        if payload[..4] != grid.rows.to_le_bytes() || payload[4..8] != grid.columns.to_le_bytes() {
            return Err(RelationalContentDenial::EncodingInconsistency);
        }
        Ok(())
    }
}
pub(crate) fn format(name: &str, version: u32, alter: bool) -> RelationalContentFormat {
    let mut formats = RelationalContentFormatRegistry::default();
    let identity = CanonicalDigestId::new([1; 32]);
    formats
        .register(
            identity,
            Arc::new(Codec {
                descriptor: ContentFormat::new(name, version).unwrap(),
                alter,
            }),
        )
        .unwrap();
    formats.get(identity).unwrap().clone()
}
pub(crate) fn grid() -> Grid {
    Grid {
        rows: 2,
        columns: 2,
        samples: vec![i64::MIN, -1, 0, i64::MAX],
    }
}
