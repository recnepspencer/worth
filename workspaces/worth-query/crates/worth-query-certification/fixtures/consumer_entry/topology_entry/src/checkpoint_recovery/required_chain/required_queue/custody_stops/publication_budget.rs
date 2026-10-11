//! Measure a publication followed by no publication, independently of posture.
use super::*;

pub(super) fn measure(budget: usize) -> Attempt {
    let (application, _invalidation) = limited_application(budget, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c) = setup_chain!(application, request);
    change_root_input!(request, application, 2, 0x9176_4080_u64);
    let source_commit = request.retain_read().unwrap().selected_commit().clone();
    drop((a, b));
    match c.advance(&request) {
        Ok(_) => Attempt::Above("last consumer refresh admitted"),
        Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded =>
        {
            if request.retain_read().unwrap().selected_commit() == &source_commit {
                Attempt::Below("refused before first upstream publication")
            } else {
                let published = request.retain_read().unwrap().selected_commit().clone();
                match c.advance(&request) {
                    Ok(_) => Attempt::Above("later consumer refresh admitted"),
                    Err(WorthQueryApplicationOutputDemandDenial::Demand(next))
                        if next.kind()
                            == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded =>
                    {
                        if request.retain_read().unwrap().selected_commit() == &published {
                            Attempt::Hit
                        } else {
                            Attempt::Above("later upstream publication admitted")
                        }
                    }
                    Err(other) => {
                        panic!("second calibration advance meets an unexpected stop: {other:?}")
                    }
                }
            }
        }
        Err(other) => panic!("publication calibration meets an unexpected stop: {other:?}"),
    }
}
