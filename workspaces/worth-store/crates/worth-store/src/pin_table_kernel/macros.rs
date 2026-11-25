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
macro_rules! verus_method {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body $body:block
    ) => {
        verus! {
            impl PinTableKernel {
                $(#[$fn_meta])*
                $vis fn $name ( $($args)* ) -> ($res: $ret)
                    $(requires $($req)*)?
                    $(ensures $($ens)*)?
                $body
            }
        }
    };
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body_with_ok_proof { $($stmts:tt)* }
        ok_proof { $($pf:tt)* }
        ok_value { $ok:expr }
    ) => {
        verus! {
            impl PinTableKernel {
                $(#[$fn_meta])*
                $vis fn $name ( $($args)* ) -> ($res: $ret)
                    $(requires $($req)*)?
                    $(ensures $($ens)*)?
                {
                    $($stmts)*
                    proof { $($pf)* }
                    $ok
                }
            }
        }
    };
}

#[cfg(not(verus_keep_ghost))]
macro_rules! verus_method {
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body $body:block
    ) => {
        impl PinTableKernel {
            $(#[$fn_meta])*
            $vis fn $name ( $($args)* ) -> $ret
            $body
        }
    };
    (
        $(#[$fn_meta:meta])*
        $vis:vis fn $name:ident ( $($args:tt)* ) -> ($res:ident : $ret:ty);
        $(requires { $($req:tt)* })?
        $(ensures { $($ens:tt)* })?
        body_with_ok_proof { $($stmts:tt)* }
        ok_proof { $($pf:tt)* }
        ok_value { $ok:expr }
    ) => {
        impl PinTableKernel {
            $(#[$fn_meta])*
            $vis fn $name ( $($args)* ) -> $ret {
                $($stmts)*
                $ok
            }
        }
    };
}

#[cfg(verus_keep_ghost)]
macro_rules! verus_scan_method {
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
            impl PinTableKernel {
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
        }
    };
}

#[cfg(not(verus_keep_ghost))]
macro_rules! verus_scan_method {
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
        impl PinTableKernel {
            $(#[$fn_meta])*
            $vis fn $name ( $($args)* ) -> $ret {
                $($init)*
                while $cond
                $step
                $($after)*
            }
        }
    };
}
