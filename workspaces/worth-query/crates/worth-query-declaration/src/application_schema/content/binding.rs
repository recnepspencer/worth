use worth_foundational::facade::CanonicalDigestId;

/// Descriptive portable meaning. This record cannot construct an installed codec.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContentFormatDeclaration {
    pub identity: [u8; 32],
    pub name: String,
    pub version: u32,
    pub maximum_frame_bytes: u64,
}

/// Callback outcomes; the runtime preserves its exact original admission stop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationContentCodecDenial {
    AdmissionStopped,
    InputTypeMismatch,
    NonCanonicalContent,
    EncodingInconsistency,
    ArithmeticOverflow,
    SizeLimit { observed: usize, maximum: usize },
}

/// Borrowed controls from the actual Native admission, with no request constructor.
pub trait ApplicationContentControl {
    fn checkpoint(&mut self, units: u64) -> Result<(), ApplicationContentCodecDenial>;
}

/// A bounded output sink and its original control are one exclusive borrow.
pub trait ApplicationContentEncoder: ApplicationContentControl {
    fn append(&mut self, bytes: &[u8]) -> Result<(), ApplicationContentCodecDenial>;
}

/// A marker declares one immutable, identity-free numerical format.
/// Correspondence verification must be independent of encoding and parsing.
pub trait ApplicationContentBinding: Copy + Send + Sync + 'static {
    type Value: Send + Sync + 'static;
    const IDENTITY: CanonicalDigestId;
    const NAME: &'static str;
    const VERSION: u32;
    const MAXIMUM_FRAME_BYTES: usize;
    fn payload_length(
        value: &Self::Value,
        control: &mut dyn ApplicationContentControl,
    ) -> Result<usize, ApplicationContentCodecDenial>;
    fn encode(
        value: &Self::Value,
        encoder: &mut dyn ApplicationContentEncoder,
    ) -> Result<(), ApplicationContentCodecDenial>;
    fn validate(
        payload: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), ApplicationContentCodecDenial>;
    fn verify_encoding(
        value: &Self::Value,
        payload: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), ApplicationContentCodecDenial>;
}
