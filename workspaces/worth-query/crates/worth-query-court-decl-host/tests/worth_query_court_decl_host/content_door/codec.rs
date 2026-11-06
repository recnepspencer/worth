//! Exact row-major signed samples; correspondence reads numeric positions independently.
use super::*;
use ApplicationContentCodecDenial as Denial;
#[derive(Clone, Copy, Debug)]
pub struct GridFormat;
#[derive(Clone)]
pub struct Grid {
    pub rows: u32,
    pub columns: u32,
    pub samples: Vec<i64>,
    pub corrupt_result: bool,
    pub swallow_sink_refusal: bool,
    pub cancel_during_encoding: Option<WorthQueryCancellationSource>,
}
impl ApplicationContentBinding for GridFormat {
    type Value = Grid;
    const IDENTITY: CanonicalDigestId = CanonicalDigestId::new([91; 32]);
    const NAME: &'static str = "exact-sample-grid";
    const VERSION: u32 = 1;
    const MAXIMUM_FRAME_BYTES: usize = 16 * 1024 * 1024;
    fn payload_length(
        value: &Grid,
        control: &mut dyn ApplicationContentControl,
    ) -> Result<usize, Denial> {
        control.checkpoint(1)?;
        if value.rows == 0
            || value.columns == 0
            || u64::from(value.rows) * u64::from(value.columns) != value.samples.len() as u64
        {
            return Err(Denial::NonCanonicalContent);
        }
        value
            .samples
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(8))
            .ok_or(Denial::ArithmeticOverflow)
    }
    fn encode(value: &Grid, encoder: &mut dyn ApplicationContentEncoder) -> Result<(), Denial> {
        if let Some(cancel) = &value.cancel_during_encoding {
            cancel.cancel();
            let _refused = encoder.checkpoint(1);
        }
        encoder.append(&value.rows.to_le_bytes())?;
        encoder.append(&value.columns.to_le_bytes())?;
        for sample in &value.samples {
            encoder.append(
                &sample
                    .wrapping_add(i64::from(value.corrupt_result))
                    .to_le_bytes(),
            )?;
        }
        if value.swallow_sink_refusal {
            let _refused = encoder.append(&[0]);
        }
        Ok(())
    }
    fn validate(payload: &[u8], control: &mut dyn ApplicationContentControl) -> Result<(), Denial> {
        control.checkpoint(payload.len() as u64)?;
        let dims = payload.get(..8).ok_or(Denial::NonCanonicalContent)?;
        let rows = u32::from_le_bytes(dims[..4].try_into().unwrap());
        let columns = u32::from_le_bytes(dims[4..].try_into().unwrap());
        if rows == 0
            || columns == 0
            || u128::from(rows) * u128::from(columns) * 8 + 8 != payload.len() as u128
        {
            return Err(Denial::NonCanonicalContent);
        }
        Ok(())
    }
    fn verify_encoding(
        value: &Grid,
        payload: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), Denial> {
        // Numerical inspection, without calling encode or validate.
        let rows = payload.get(..4).ok_or(Denial::EncodingInconsistency)?;
        let columns = payload.get(4..8).ok_or(Denial::EncodingInconsistency)?;
        if u32::from_le_bytes(rows.try_into().unwrap()) != value.rows
            || u32::from_le_bytes(columns.try_into().unwrap()) != value.columns
        {
            return Err(Denial::EncodingInconsistency);
        }
        for (index, sample) in value.samples.iter().enumerate() {
            control.checkpoint(8)?;
            let word = payload
                .get(8 + index * 8..16 + index * 8)
                .ok_or(Denial::EncodingInconsistency)?;
            if i64::from_le_bytes(word.try_into().unwrap()) != *sample {
                return Err(Denial::EncodingInconsistency);
            }
        }
        Ok(())
    }
}

// Each describes a different field interpretation over the same ordered grid payload.
macro_rules! format_marker {
    ($marker:ident,$id:expr,$name:expr,$version:expr,$maximum:expr) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $marker;
        impl ApplicationContentBinding for $marker {
            type Value = Grid;
            const IDENTITY: CanonicalDigestId = CanonicalDigestId::new([$id; 32]);
            const NAME: &'static str = $name;
            const VERSION: u32 = $version;
            const MAXIMUM_FRAME_BYTES: usize = $maximum;
            fn payload_length(
                value: &Grid,
                control: &mut dyn ApplicationContentControl,
            ) -> Result<usize, Denial> {
                GridFormat::payload_length(value, control)
            }
            fn encode(
                value: &Grid,
                encoder: &mut dyn ApplicationContentEncoder,
            ) -> Result<(), Denial> {
                GridFormat::encode(value, encoder)
            }
            fn validate(
                payload: &[u8],
                control: &mut dyn ApplicationContentControl,
            ) -> Result<(), Denial> {
                GridFormat::validate(payload, control)
            }
            fn verify_encoding(
                value: &Grid,
                payload: &[u8],
                control: &mut dyn ApplicationContentControl,
            ) -> Result<(), Denial> {
                GridFormat::verify_encoding(value, payload, control)
            }
        }
    };
}
format_marker!(OtherGrid, 92, "other-sample-grid", 1, 16 * 1024 * 1024);
format_marker!(GridV2, 93, "exact-sample-grid", 2, 16 * 1024 * 1024);
format_marker!(Impostor, 91, "exact-sample-grid", 1, 16 * 1024 * 1024);
