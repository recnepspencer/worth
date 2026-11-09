use super::*;
impl<const F: usize> Fragment for Records<18, F, false> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        (
            Record<0, F>,
            Record<1, F>,
            Record<2, F>,
            Record<3, F>,
            Record<4, F>,
            Record<5, F>,
            Record<6, F>,
        ),
        (
            Record<7, F>,
            Record<8, F>,
            Record<9, F>,
            Record<10, F>,
            Record<11, F>,
            Record<12, F>,
            Record<13, F>,
        ),
        (
            Record<14, F>,
            Record<15, F>,
            Record<16, F>,
            Record<17, F>,
            Contracts<F, false>,
        ),
    );
}
impl<const F: usize> Fragment for Records<18, F, true> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        (
            Record<17, F>,
            Record<16, F>,
            Record<15, F>,
            Record<14, F>,
            Record<13, F>,
            Record<12, F>,
            Record<11, F>,
        ),
        (
            Record<10, F>,
            Record<9, F>,
            Record<8, F>,
            Record<7, F>,
            Record<6, F>,
            Record<5, F>,
            Record<4, F>,
        ),
        (
            Record<3, F>,
            Record<2, F>,
            Record<1, F>,
            Record<0, F>,
            Contracts<F, true>,
        ),
    );
}
pub struct Contracts<const F: usize, const REVERSE: bool>;
#[path = "count_18/forward.rs"]
mod forward;
#[path = "count_18/reverse.rs"]
mod reverse;
