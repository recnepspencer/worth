// Keep the media owner alive through request admission, as in the original court.
#[macro_export]
macro_rules! initialize_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::physical_record_journeys::durability(&media_owner);
        media_owner.initialize_record_store($request)
    }};
}

#[macro_export]
macro_rules! open_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::physical_record_journeys::durability(&media_owner);
        media_owner.open_record_store($request)
    }};
}
