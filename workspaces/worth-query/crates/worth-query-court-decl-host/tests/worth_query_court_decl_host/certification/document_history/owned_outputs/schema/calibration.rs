//! An optional recorded decision input, initialized by the real Preserve producer.
use super::*;
pub struct Calibration;
impl Aspect for Calibration {
    const NAME: &'static str = "Calibration";
    type Row = CalibrationRow;
}
impl Calibration {
    pub fn reference() -> AspectSelection<Storage, Member, Self> {
        aspect(Storage, Member, Self)
    }
}
pub struct CalibrationRow {
    pub ready: u64,
}
impl Record<CalibrationRow> for Bindings {
    fn record() -> impl Shape<CalibrationRow, Self> {
        field("Ready", |row: &CalibrationRow| row.ready)
            .equality_queryable(true)
            .build(|ready, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CalibrationRow { ready })
            })
    }
}
pub type CalibrationReady = FieldSelection<Storage, Member, Calibration, u64>;
