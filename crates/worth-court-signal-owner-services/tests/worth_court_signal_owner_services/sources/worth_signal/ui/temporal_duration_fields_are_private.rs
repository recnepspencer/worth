use worth_signal::facade::TemporalDuration;

fn main() {
    let _duration = TemporalDuration {
        milliseconds: std::num::NonZeroU64::new(5).unwrap(),
        _tag: std::marker::PhantomData,
    };
}
