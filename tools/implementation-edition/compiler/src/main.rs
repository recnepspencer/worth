//! Pinned compiler callback. No consumer-facing edition construction lives here.
#![feature(rustc_private)]
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod compiler_arguments;
mod compiler_identity;
mod expanded_inputs;
mod returned_artifacts;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use std::path::PathBuf;

#[derive(Default)]
struct Observer {
    audited: bool,
    expanded: String,
    files: Vec<PathBuf>,
}
impl Callbacks for Observer {
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        if tcx
            .crate_types()
            .iter()
            .any(|kind| *kind != rustc_session::config::CrateType::Rlib)
        {
            compiler
                .sess
                .dcx()
                .fatal("actual crate type requires unit-context qualification");
        }
        let (_, krate) = &*tcx.resolver_for_lowering().borrow();
        if let Err(reason) = expanded_inputs::check(krate) {
            compiler.sess.dcx().fatal(reason);
        }
        self.audited = true;
        self.expanded = returned_artifacts::digest(format!("{krate:?}").as_bytes());
        let outputs = tcx.output_filenames(());
        for kind in compiler.sess.opts.output_types.keys() {
            if *kind == rustc_session::config::OutputType::Exe {
                for crate_type in tcx.crate_types() {
                    if let rustc_session::config::OutFileName::Real(path) =
                        rustc_session::output::filename_for_input(
                            &compiler.sess,
                            *crate_type,
                            tcx.crate_name(rustc_span::def_id::LOCAL_CRATE),
                            outputs,
                        )
                    {
                        self.files.push(path);
                    }
                }
            } else if let rustc_session::config::OutFileName::Real(path) = outputs.path(*kind) {
                self.files.push(path);
            }
        }
        Compilation::Continue
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let (identity, sysroot) = compiler_identity::verified().unwrap_or_else(|reason| {
        eprintln!("implementation-edition compiler: {reason}");
        std::process::exit(1);
    });
    if let Err(reason) = compiler_arguments::check(&args[1..]) {
        eprintln!("implementation-edition compiler: {reason}");
        std::process::exit(1);
    }
    args.push(format!("--sysroot={}", sysroot.display()));
    if args.iter().any(|arg| arg == "-vV") {
        // sccache's compiler identity includes observer and pinned runtime.
        // This cannot share a compiler key with unaudited vanilla rustc.
        rustc_driver::run_compiler(&args, &mut Observer::default());
        println!("implementation-observer: {identity}");
        return;
    }
    let mut observer = Observer::default();
    rustc_driver::run_compiler(&args, &mut observer);
    if observer.audited
        && let Err(reason) =
            returned_artifacts::bind(&observer.files, &observer.expanded, &identity)
    {
        eprintln!("implementation-edition compiler: {reason}");
        std::process::exit(1);
    }
}
