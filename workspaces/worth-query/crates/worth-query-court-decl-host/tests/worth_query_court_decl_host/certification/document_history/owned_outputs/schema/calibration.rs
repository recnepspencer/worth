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

/// Edit-envelope content, independent of producer-owned calibration readiness.
pub struct EnvelopeProgress;
impl Aspect for EnvelopeProgress {
    const NAME: &'static str = "EnvelopeProgress";
    type Row = EnvelopeProgressRow;
}
pub struct EnvelopeProgressRow {
    pub dependent: u64,
    pub generation: u64,
    pub frontier: u64,
}
impl Record<EnvelopeProgressRow> for Bindings {
    fn record() -> impl Shape<EnvelopeProgressRow, Self> {
        field("Dependent", |row: &EnvelopeProgressRow| row.dependent)
            .equality_queryable(true)
            .and("Generation", |row: &EnvelopeProgressRow| row.generation)
            .equality_queryable(true)
            .and("Frontier", |row: &EnvelopeProgressRow| row.frontier)
            .equality_queryable(true)
            .build(
                |dependent, generation, frontier, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(EnvelopeProgressRow {
                        dependent,
                        generation,
                        frontier,
                    })
                },
            )
    }
}
pub type EnvelopeDependent = FieldSelection<Storage, Member, EnvelopeProgress, u64>;
pub type EnvelopeGeneration = FieldSelection<Storage, Member, EnvelopeProgress, u64>;
pub type EnvelopeFrontier = FieldSelection<Storage, Member, EnvelopeProgress, u64>;
