use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalCertifiedPerformanceBundle,
    FoundationalCounterBackedPerformanceReceipt,
};

fn main() {
    let _ = FoundationalCertifiedPerformanceBundle::<FoundationalCounterBackedPerformanceReceipt<
        FoundationalAuthoritativePerformanceClaim,
    >> {
        inner: impossible_inner(),
    };
}

fn impossible_inner() -> ! {
    loop {
        std::hint::spin_loop();
    }
}
