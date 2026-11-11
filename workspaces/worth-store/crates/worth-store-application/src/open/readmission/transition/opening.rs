//! Consume the original admitted owner decision before Native construction.
use crate::AdmittedApplicationHome;
use worth_proof::HomeAdmitted;

pub enum ApplicationHomeProgramOpening {
    Current(HomeAdmitted<AdmittedApplicationHome>),
    TransitionRequired(HomeAdmitted<AdmittedApplicationHome>),
}

pub fn select_home_program_opening(
    home: HomeAdmitted<AdmittedApplicationHome>,
) -> ApplicationHomeProgramOpening {
    home.consume(|home| {
        let required = home.context.transition.required().is_some();
        let home = HomeAdmitted::from_owner(home);
        if required {
            ApplicationHomeProgramOpening::TransitionRequired(home)
        } else {
            ApplicationHomeProgramOpening::Current(home)
        }
    })
}
