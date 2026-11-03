pub(crate) mod nested;

pub struct Opened;
pub(crate) struct Hidden;

#[cfg(test)]
pub(crate) fn hook() {}

pub(crate) fn helper(value: u8) -> u8 {
    value
}

impl Opened {
    pub fn real(&self) -> u8 {
        1
    }

    #[cfg(test)]
    pub(crate) fn probe_for_test(&self) -> u8 {
        2
    }
}

#[cfg(not(test))]
mod production_only {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_returns_its_input() {
        assert_eq!(helper(1), 1);
        let _hidden = Hidden;
    }
}

#[cfg(test)]
mod public_tests {
    use crate::facade::{Globbed, Opened};

    #[test]
    fn opened_and_globbed_are_public() {
        let _ = (Opened.real(), Globbed);
    }
}

#[cfg(all(test, target_pointer_width = "64"))]
mod hook_tests {
    macro_rules! twice {
        ($name:ident) => {
            #[test]
            fn $name() {}
        };
    }
    twice!(generated_case);

    #[test]
    fn hook_and_probe_are_test_only() {
        super::hook();
        let _ = super::Opened.probe_for_test();
        crate::world::build();
    }
}
