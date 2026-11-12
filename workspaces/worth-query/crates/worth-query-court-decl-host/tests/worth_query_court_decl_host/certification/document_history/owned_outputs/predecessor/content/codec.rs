//! Fixed-width word content; independent equality checks the authored value.
use super::*;
use ApplicationContentCodecDenial as Denial;
#[derive(Clone, Copy)]
pub struct Word;
impl ApplicationContentBinding for Word {
    type Value = u64;
    const IDENTITY: CanonicalDigestId = CanonicalDigestId::new([157; 32]);
    const NAME: &'static str = "predecessor-word";
    const VERSION: u32 = 1;
    // The bound includes the versioned content envelope as well as eight payload bytes.
    const MAXIMUM_FRAME_BYTES: usize = 128;
    fn payload_length(
        _: &u64,
        control: &mut dyn ApplicationContentControl,
    ) -> Result<usize, Denial> {
        control.checkpoint(1)?;
        Ok(8)
    }
    fn encode(value: &u64, encoder: &mut dyn ApplicationContentEncoder) -> Result<(), Denial> {
        encoder.append(&value.to_le_bytes())
    }
    fn validate(bytes: &[u8], control: &mut dyn ApplicationContentControl) -> Result<(), Denial> {
        control.checkpoint(bytes.len() as u64)?;
        if bytes.len() == 8 {
            Ok(())
        } else {
            Err(Denial::NonCanonicalContent)
        }
    }
    fn verify_encoding(
        value: &u64,
        bytes: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), Denial> {
        control.checkpoint(8)?;
        if bytes == value.to_le_bytes() {
            Ok(())
        } else {
            Err(Denial::EncodingInconsistency)
        }
    }
}
