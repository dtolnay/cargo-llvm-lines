//! [![github]](https://github.com/dtolnay/cargo-llvm-lines)&ensp;[![crates-io]](https://crates.io/crates/cargo-llvm-lines)&ensp;[![docs-rs]](https://docs.rs/cargo-llvm-lines)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

#![allow(
    clippy::cast_precision_loss,
    clippy::let_underscore_drop,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::uninlined_format_args,
    clippy::unseparated_literal_suffix
)]

mod count;
mod opts;
mod table;

use crate::count::{count_lines, Instantiations};
use crate::opts::{LlvmLines, SortOrder, Subcommand};
use atty::Stream::Stderr;
use clap::{CommandFactory, Parser};
use regex::Regex;
use std::collections::HashMap as Map;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use tempdir::TempDir;

cargo_subcommand_metadata::description!(
    "Count the number of lines of LLVM IR across all instantiations of a generic function"
);

fn main() {
    let Subcommand::LlvmLines(opts) = Subcommand::parse();

    if opts.help {
        let _ = Subcommand::command()
            .get_subcommands_mut()
            .next()
            .unwrap()
            .print_help();
        return;
    }

    if opts.version {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(Subcommand::command().render_version().as_bytes());
        return;
    }

    // If `--filter-cargo` was specified, just filter the output and exit early.
    let result = if opts.filter_cargo {
        filter_err();
        Ok(0)
    } else if opts.files.is_empty() {
        cargo_llvm_lines(&opts)
    } else {
        read_llvm_ir_from_paths(&opts.files, opts.sort, opts.filter.as_ref())
    };

    process::exit(match result {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(io::stderr(), "{}", err);
            1
        }
    });
}

fn cargo_llvm_lines(opts: &LlvmLines) -> io::Result<i32> {
    let outdir = TempDir::new("cargo-llvm-lines").expect("failed to create tmp file");
    let outfile = outdir.path().join("crate");

    let exit = run_cargo_rustc(opts, &outfile)?;
    if exit != 0 {
        return Ok(exit);
    }

    let ir = read_llvm_ir_from_dir(&outdir)?;
    let mut instantiations = Map::<String, Instantiations>::new();
    count_lines(&mut instantiations, &ir);
    table::print(instantiations, opts.sort, opts.filter.as_ref());

    Ok(0)
}

fn run_cargo_rustc(opts: &LlvmLines, outfile: &Path) -> io::Result<i32> {
    // If cargo-llvm-lines was invoked from cargo, use the cargo that invoked it.
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cmd = Command::new(cargo);
    propagate_opts(&mut cmd, opts, outfile);
    cmd.env("CARGO_INCREMENTAL", "");
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn()?;

    // Filter stderr through a second invocation of `cargo-llvm-lines` that has
    // `--filter-cargo` specified so that it just does the filtering in
    // `filter_err()` above.
    let current_exe = env::current_exe()?;
    let mut errcmd = Command::new(current_exe);
    errcmd.arg("llvm-lines");
    errcmd.arg("--filter-cargo");
    errcmd.stdin(child.stderr.take().ok_or(io::ErrorKind::BrokenPipe)?);
    errcmd.stdout(Stdio::null());
    errcmd.stderr(Stdio::inherit());
    let mut errchild = errcmd.spawn()?;

    errchild.wait()?;
    child.wait().map(|status| status.code().unwrap_or(1))
}

fn read_llvm_ir_from_dir(outdir: &TempDir) -> io::Result<Vec<u8>> {
    for file in fs::read_dir(outdir)? {
        let path = file?.path();
        if let Some(ext) = path.extension() {
            if ext == "ll" {
                return fs::read(path);
            }
        }
    }

    let msg = "Ran --emit=llvm-ir but did not find output IR";
    Err(io::Error::new(ErrorKind::Other, msg))
}

fn read_llvm_ir_from_paths(
    paths: &[PathBuf],
    sort_order: SortOrder,
    function_filter: Option<&Regex>,
) -> io::Result<i32> {
    let mut instantiations = Map::<String, Instantiations>::new();

    for path in paths {
        match fs::read(path) {
            Ok(ir) => count_lines(&mut instantiations, &ir),
            Err(err) => {
                let msg = format!("{}: {}", path.display(), err);
                return Err(io::Error::new(err.kind(), msg));
            }
        }
    }

    table::print(instantiations, sort_order, function_filter);
    Ok(0)
}

// Based on https://github.com/rsolomo/cargo-check
fn propagate_opts(cmd: &mut Command, opts: &LlvmLines, outfile: &Path) {
    let LlvmLines {
        // Strip out options that are for cargo-llvm-lines itself.
        sort: _,
        filter: _,
        files: _,
        filter_cargo: _,
        help: _,
        version: _,

        // Options to pass through to the cargo rustc invocation.
        quiet,
        ref package,
        lib,
        ref bin,
        ref example,
        ref test,
        release,
        ref profile,
        ref features,
        all_features,
        no_default_features,
        ref target,
        ref manifest_path,
        ref rest,
    } = *opts;

    cmd.arg("rustc");

    if quiet {
        cmd.arg("--quiet");
    }

    if let Some(package) = package {
        cmd.arg("--package");
        cmd.arg(package);
    }

    if lib {
        cmd.arg("--lib");
    }

    if let Some(bin) = bin {
        cmd.arg("--bin");
        cmd.arg(bin);
    }

    if let Some(example) = example {
        cmd.arg("--example");
        cmd.arg(example);
    }

    if let Some(test) = test {
        cmd.arg("--test");
        cmd.arg(test);
    }

    if release {
        cmd.arg("--release");
    }

    if let Some(profile) = profile {
        cmd.arg("--profile");
        cmd.arg(profile);
    }

    if let Some(features) = features {
        cmd.arg("--features");
        cmd.arg(features);
    }

    if all_features {
        cmd.arg("--all-features");
    }

    if no_default_features {
        cmd.arg("--no-default-features");
    }

    if let Some(target) = target {
        cmd.arg("--target");
        cmd.arg(target);
    }

    if let Some(manifest_path) = manifest_path {
        cmd.arg("--manifest-path");
        cmd.arg(manifest_path);
    }

    let color = atty::is(Stderr);
    let setting = if color { "always" } else { "never" };
    cmd.arg(format!("--color={}", setting));

    // The `-Cno-prepopulate-passes` means we skip LLVM optimizations, which is
    // good because (a) we count the LLVM IR lines are sent to LLVM, not how
    // many there are after optimizations run, and (b) it's faster.
    //
    // The `-Cpasses=name-anon-globals` follows on: it's required to avoid the
    // following error on some programs: "The current compilation is going to
    // use thin LTO buffers without running LLVM's NameAnonGlobals pass. This
    // will likely cause errors in LLVM. Consider adding -C
    // passes=name-anon-globals to the compiler command line."
    cmd.arg("--");
    cmd.arg("--emit=llvm-ir");
    cmd.arg("-Cno-prepopulate-passes");
    cmd.arg("-Cpasses=name-anon-globals");
    cmd.arg("-o");
    cmd.arg(outfile);
    cmd.args(rest);
}

/// Print lines from stdin to stderr, skipping lines that `ignore` succeeds on.
fn filter_err() {
    let mut line = String::new();
    while let Ok(n) = io::stdin().read_line(&mut line) {
        if n == 0 {
            break;
        }
        if !ignore_cargo_err(&line) {
            let _ = write!(io::stderr(), "{}", line);
        }
        line.clear();
    }
}

/// Match Cargo output lines that we don't want to be printed.
fn ignore_cargo_err(line: &str) -> bool {
    if line.trim().is_empty() {
        return true;
    }

    let discarded_lines = [
        "warnings emitted",
        "ignoring specified output filename because multiple outputs were \
         requested",
        "ignoring specified output filename for 'link' output because multiple \
         outputs were requested",
        "ignoring --out-dir flag due to -o flag",
        "due to multiple output types requested, the explicitly specified \
         output file name will be adapted for each output type",
        "ignoring -C extra-filename flag due to -o flag",
    ];
    for s in &discarded_lines {
        if line.contains(s) {
            return true;
        }
    }

    // warning: `cratename` (lib) generated 2 warnings
    if let Some(i) = line.find(") generated ") {
        let rest = &line[i + ") generated ".len()..];
        let n = rest.bytes().take_while(u8::is_ascii_digit).count();
        if n > 0 && rest[n..].starts_with(" warning") {
            return true;
        }
    }

    false
}
