use super::*;
macro_rules! format {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub(crate) struct $name;
        impl ApplicationContentBinding for $name {
            type Value = u64;
            const IDENTITY: CanonicalDigestId = CanonicalDigestId::new([31; 32]);
            const NAME: &'static str = "exact-word";
            const VERSION: u32 = 1;
            const MAXIMUM_FRAME_BYTES: usize = 64;
            fn payload_length(
                _: &u64,
                c: &mut dyn ApplicationContentControl,
            ) -> Result<usize, ApplicationContentCodecDenial> {
                c.checkpoint(1)?;
                Ok(8)
            }
            fn encode(
                v: &u64,
                c: &mut dyn ApplicationContentEncoder,
            ) -> Result<(), ApplicationContentCodecDenial> {
                c.append(&v.to_le_bytes())
            }
            fn validate(
                p: &[u8],
                c: &mut dyn ApplicationContentControl,
            ) -> Result<(), ApplicationContentCodecDenial> {
                c.checkpoint(8)?;
                if p.len() == 8 {
                    Ok(())
                } else {
                    Err(ApplicationContentCodecDenial::NonCanonicalContent)
                }
            }
            fn verify_encoding(
                v: &u64,
                p: &[u8],
                c: &mut dyn ApplicationContentControl,
            ) -> Result<(), ApplicationContentCodecDenial> {
                c.checkpoint(8)?;
                let bytes: [u8; 8] = p
                    .try_into()
                    .map_err(|_| ApplicationContentCodecDenial::EncodingInconsistency)?;
                if u64::from_le_bytes(bytes) == *v {
                    Ok(())
                } else {
                    Err(ApplicationContentCodecDenial::EncodingInconsistency)
                }
            }
        }
    };
}
format!(Good);
format!(Impostor);
