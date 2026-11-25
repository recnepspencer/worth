//! A fragment beyond the old eight-member cap lowers every member once, in order.
use crate::application_schema::fragment::{self, Fragment, NoBindings, Registration};
use crate::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};
use std::cell::RefCell;

thread_local! {
    static LOWERED: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy)]
struct WideSchema;

impl ApplicationSchema for WideSchema {
    const OWNER: &'static str = "WORTH.tests.wide-fragment";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}

macro_rules! members {
    ($($member:ident = $index:literal),+) => {
        $(
            struct $member;
            impl Registration<WideSchema> for $member {
                fn register(
                    builder: ApplicationSchemaDeclarationBuilder<WideSchema>,
                ) -> Result<
                    ApplicationSchemaDeclarationBuilder<WideSchema>,
                    ApplicationSchemaDeclarationDenial,
                > {
                    LOWERED.with(|lowered| lowered.borrow_mut().push($index));
                    Ok(builder)
                }
            }
        )+
        impl Fragment for WideSchema {
            const NAME: &'static str = "WideSchema";
            type Bindings = NoBindings;
            type Members = ($($member,)+);
        }
    };
}

members!(
    M1 = 1,
    M2 = 2,
    M3 = 3,
    M4 = 4,
    M5 = 5,
    M6 = 6,
    M7 = 7,
    M8 = 8,
    M9 = 9,
    M10 = 10,
    M11 = 11,
    M12 = 12
);

#[test]
fn a_twelve_member_fragment_registers_every_member_in_order() {
    LOWERED.with(|lowered| lowered.borrow_mut().clear());
    assert!(fragment::declaration_builder(WideSchema).is_ok());
    let lowered = LOWERED.with(|lowered| lowered.borrow().clone());
    assert_eq!(lowered, (1..=12).collect::<Vec<_>>());
}
