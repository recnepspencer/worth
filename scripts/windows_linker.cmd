@echo off
rem Cargo's target-scoped x86_64 Windows MSVC linker (see .cargo/config.toml).
rem Forwards every argument to rust-lld unchanged and lets each PDB grow past
rem the 4 GiB default page-size limit. rustc puts its sysroot tools, including
rem rust-lld, on this process's PATH, and passes a response file when the
rem command line is long; lld-link reads it.
rust-lld -flavor link %* /pdbpagesize:16384
