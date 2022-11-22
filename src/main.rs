//! [![github]](https://github.com/dtolnay/cargo-llvm-lines)&ensp;[![crates-io]](https://crates.io/crates/cargo-llvm-lines)&ensp;[![docs-rs]](https://docs.rs/cargo-llvm-lines)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

#![allow(
    clippy::cast_precision_loss,
    clippy::let_underscore_drop,
    clippy::uninlined_format_args,
    clippy::unseparated_literal_suffix
)]

mod opts;

use crate::opts::{SortOrder, Subcommand};
use atty::Stream::Stderr;
use clap::{CommandFactory, Parser};
use regex::Regex;
use rustc_demangle::demangle;
use std::collections::HashMap as Map;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use tempdir::TempDir;

cargo_subcommand_metadata::description!(
    "Count the number of lines of LLVM IR across all instantiations of a generic function"
);

fn main() {
    let Subcommand::LlvmLines {
        filter_cargo,
        sort,
        filter: function_filter,
        files,
        help,
        version,
        ..
    } = Subcommand::parse();

    if help {
        let _ = Subcommand::command()
            .get_subcommands_mut()
            .next()
            .unwrap()
            .print_help();
        return;
    }

    if version {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(Subcommand::command().render_version().as_bytes());
        return;
    }

    let result = if files.is_empty() {
        cargo_llvm_lines(filter_cargo, sort, function_filter.as_ref())
    } else {
        read_llvm_ir_from_paths(&files, sort, function_filter.as_ref())
    };

    process::exit(match result {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(io::stderr(), "{}", err);
            1
        }
    });
}

fn cargo_llvm_lines(
    filter_cargo: bool,
    sort_order: SortOrder,
    function_filter: Option<&Regex>,
) -> io::Result<i32> {
    // If `--filter-cargo` was specified, just filter the output and exit
    // early.
    if filter_cargo {
        filter_err(ignore_cargo_err);
    }

    let outdir = TempDir::new("cargo-llvm-lines").expect("failed to create tmp file");
    let outfile = outdir.path().join("crate");

    let exit = run_cargo_rustc(&outfile)?;
    if exit != 0 {
        return Ok(exit);
    }

    let ir = read_llvm_ir_from_dir(&outdir)?;
    let mut instantiations = Map::<String, Instantiations>::new();
    count_lines(&mut instantiations, &ir);
    print_table(instantiations, sort_order, function_filter);

    Ok(0)
}

fn run_cargo_rustc(outfile: &Path) -> io::Result<i32> {
    // If cargo-llvm-lines was invoked from cargo, use the cargo that invoked it.
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cmd = Command::new(cargo);

    // Strip out options that are for cargo-llvm-lines itself.
    let mut prev_was_filter = false;
    let args: Vec<_> = env::args_os()
        .filter(|s| {
            let x = s.to_string_lossy();
            if x == "--filter" {
                prev_was_filter = true;
                return false;
            } else if prev_was_filter {
                prev_was_filter = false;
                return false;
            }
            !["--sort", "-s", "lines", "copies"].contains(&x.as_ref())
        })
        .collect();
    cmd.args(&wrap_args(args.clone(), outfile));

    cmd.env("CARGO_INCREMENTAL", "");
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn()?;

    // Duplicate the original command, and insert `--filter-cargo` just after
    // the `cargo-llvm-lines` and `llvm-lines` arguments.
    //
    // Note: the `--filter-cargo` must be inserted there, rather than appended
    // to the end, so that it comes before a possible `--` arguments. Otherwise
    // it will be ignored by the recursive invocation.
    let mut filter_cargo = Vec::new();
    filter_cargo.extend(args.iter().map(OsString::as_os_str));
    filter_cargo.insert(2, OsStr::new("--filter-cargo"));

    // Filter stderr through a second invocation of `cargo-llvm-lines` that has
    // `--filter-cargo` specified so that it just does the filtering in
    // `filter_err()` above.
    let mut errcmd = Command::new(filter_cargo[0]);
    errcmd.args(&filter_cargo[1..]);
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

    print_table(instantiations, sort_order, function_filter);
    Ok(0)
}

#[derive(Default)]
struct Instantiations {
    copies: usize,
    total_lines: usize,
}

impl Instantiations {
    fn record_lines(&mut self, lines: usize) {
        self.copies += 1;
        self.total_lines += lines;
    }
}

fn count_lines(instantiations: &mut Map<String, Instantiations>, ir: &[u8]) {
    let mut current_function = None;
    let mut count = 0;

    for line in String::from_utf8_lossy(ir).lines() {
        if line.starts_with("define ") {
            current_function = parse_function_name(line);
        } else if line == "}" {
            if let Some(name) = current_function.take() {
                instantiations
                    .entry(name)
                    .or_insert_with(Default::default)
                    .record_lines(count);
            }
            count = 0;
        } else if line.starts_with("  ") && !line.starts_with("   ") {
            count += 1;
        }
    }
}

fn print_table(
    instantiations: Map<String, Instantiations>,
    sort_order: SortOrder,
    function_filter: Option<&Regex>,
) {
    let mut data = instantiations.into_iter().collect::<Vec<_>>();

    let mut total = Instantiations {
        copies: 0,
        total_lines: 0,
    };
    for row in &data {
        total.copies += row.1.copies;
        total.total_lines += row.1.total_lines;
    }

    match sort_order {
        SortOrder::Lines => {
            data.sort_by(|a, b| {
                let key_lo = (b.1.total_lines, b.1.copies, &a.0);
                let key_hi = (a.1.total_lines, a.1.copies, &b.0);
                key_lo.cmp(&key_hi)
            });
        }
        SortOrder::Copies => {
            data.sort_by(|a, b| {
                let key_lo = (b.1.copies, b.1.total_lines, &a.0);
                let key_hi = (a.1.copies, a.1.total_lines, &b.0);
                key_lo.cmp(&key_hi)
            });
        }
    }

    let lines_width = total.total_lines.to_string().len();
    let copies_width = total.copies.to_string().len();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(
        handle,
        "  Lines{0:1$}           Copies{0:2$}          Function name",
        "", lines_width, copies_width,
    );
    let _ = writeln!(
        handle,
        "  -----{0:1$}           ------{0:2$}          -------------",
        "", lines_width, copies_width,
    );
    let _ = writeln!(
        handle,
        "  {0:1$}                {2:3$}                (TOTAL)",
        total.total_lines, lines_width, total.copies, copies_width,
    );
    let mut cumul_lines = 0;
    let mut cumul_copies = 0;
    let perc = |m, cumul_m: &mut _, n| {
        *cumul_m += m;
        format!(
            "({:3.1}%,{:5.1}%)",
            m as f64 / n as f64 * 100f64,
            *cumul_m as f64 / n as f64 * 100f64,
        )
    };
    for row in data {
        if function_filter.map_or(true, |ff| ff.is_match(&row.0)) {
            let _ = writeln!(
                handle,
                "  {0:1$} {2:<14} {3:4$} {5:<14} {6}",
                row.1.total_lines,
                lines_width,
                perc(row.1.total_lines, &mut cumul_lines, total.total_lines),
                row.1.copies,
                copies_width,
                perc(row.1.copies, &mut cumul_copies, total.copies),
                row.0,
            );
        }
    }
}

fn parse_function_name(line: &str) -> Option<String> {
    let start = line.find('@')? + 1;
    let end = line[start..].find('(')?;
    let mangled = line[start..start + end].trim_matches('"');
    let mut name = demangle(mangled).to_string();
    if has_hash(&name) {
        let len = name.len() - 19;
        name.truncate(len);
    }
    Some(name)
}

fn has_hash(name: &str) -> bool {
    let mut bytes = name.bytes().rev();
    for _ in 0..16 {
        if !bytes.next().map_or(false, is_ascii_hexdigit) {
            return false;
        }
    }
    bytes.next() == Some(b'h') && bytes.next() == Some(b':') && bytes.next() == Some(b':')
}

fn is_ascii_hexdigit(byte: u8) -> bool {
    (b'0'..=b'9').contains(&byte) || (b'a'..=b'f').contains(&byte)
}

// Based on https://github.com/rsolomo/cargo-check
fn wrap_args<I>(it: I, outfile: &Path) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = vec!["rustc".into()];
    let mut has_color = false;

    // Skip the `cargo-llvm-lines` and `llvm-lines` arguments.
    let mut it = it.into_iter().skip(2);
    for arg in &mut it {
        if arg == *"--" {
            break;
        }
        has_color |= arg.to_str().unwrap_or("").starts_with("--color");
        args.push(arg);
    }

    if !has_color {
        let color = atty::is(Stderr);
        let setting = if color { "always" } else { "never" };
        args.push(format!("--color={}", setting).into());
    }

    // The `-Cno-prepopulate-passes` means we skip LLVM optimizations, which is
    // good because (a) we count the LLVM IR lines are sent to LLVM, not how
    // many there are after optimizations run, and (b) it's faster.
    //
    // The `-Cpasses=name-anon-globals` follows on: it's required to avoid the
    // following error on some programs: "The current compilation is going to
    // use thin LTO buffers without running LLVM's NameAnonGlobals pass. This
    // will likely cause errors in LLVM. Consider adding -C
    // passes=name-anon-globals to the compiler command line."
    args.push("--".into());
    args.push("--emit=llvm-ir".into());
    args.push("-Cno-prepopulate-passes".into());
    args.push("-Cpasses=name-anon-globals".into());
    args.push("-o".into());
    args.push(outfile.into());
    args.extend(it);
    args
}

/// Print lines from stdin to stderr, skipping lines that `ignore` succeeds on.
fn filter_err(ignore: fn(&str) -> bool) -> ! {
    let mut line = String::new();
    while let Ok(n) = io::stdin().read_line(&mut line) {
        if n == 0 {
            break;
        }
        if !ignore(&line) {
            let _ = write!(io::stderr(), "{}", line);
        }
        line.clear();
    }
    process::exit(0);
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
