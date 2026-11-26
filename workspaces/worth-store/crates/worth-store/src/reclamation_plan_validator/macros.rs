#[cfg(verus_keep_ghost)]
macro_rules! verus_items {
    ($($tt:tt)*) => {
        verus! { $($tt)* }
    };
}

#[cfg(not(verus_keep_ghost))]
macro_rules! verus_items {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(verus_keep_ghost)]
macro_rules! verus_fn {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body $body:block
    ) => {
        verus! {
            $(#[$fn_meta])*
            $vis fn $name ( $($args)* ) -> ($res: $ret)
                $(requires $($req)*)?
                $(ensures $($ens)*)?
            $body
        }
    };
}

#[cfg(not(verus_keep_ghost))]
macro_rules! verus_fn {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body $body:block
    ) => {
        $(#[$fn_meta])*
        $vis fn $name ( $($args)* ) -> $ret
        $body
    };
}

#[cfg(verus_keep_ghost)]
macro_rules! verus_scan_fn {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(ensures { $($ens:tt)* })?
        init { $($init:tt)* }
        while ($cond:expr)
        invariant { $($inv:tt)* }
        decreases { $($dec:tt)* }
        step $step:block
        after { $($after:tt)* }
    ) => {
        verus! {
            $(#[$fn_meta])*
            $vis fn $name ( $($args)* ) -> ($res: $ret)
                $(ensures $($ens)*)?
            {
                $($init)*
                while $cond
                    invariant $($inv)*
                    decreases $($dec)*
                $step
                $($after)*
            }
        }
    };
}

#[cfg(not(verus_keep_ghost))]
macro_rules! verus_scan_fn {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(ensures { $($ens:tt)* })?
        init { $($init:tt)* }
        while ($cond:expr)
        invariant { $($inv:tt)* }
        decreases { $($dec:tt)* }
        step $step:block
        after { $($after:tt)* }
    ) => {
        $(#[$fn_meta])*
        $vis fn $name ( $($args)* ) -> $ret {
            $($init)*
            while $cond
            $step
            $($after)*
        }
    };
}
