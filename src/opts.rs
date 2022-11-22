use clap::{Parser, ValueEnum};
use regex::Regex;
use std::ffi::OsString;
use std::path::PathBuf;

const ABOUT: &str = "
Print the number of lines of LLVM IR that is generated for the current project.

Options shown below without an explanation mean the same thing as the
corresponding option of `cargo build`.";

const TEMPLATE: &str = "\
{bin} {version}
{author}
{about}

{usage-heading}
    {usage}

{all-args}";

#[derive(Parser, Debug)]
#[command(
    name = "cargo-llvm-lines",
    bin_name = "cargo",
    author,
    version,
    disable_help_subcommand = true
)]
#[allow(dead_code)]
pub enum Opt {
    #[command(
        name = "llvm-lines",
        author,
        version,
        about = ABOUT,
        help_template = TEMPLATE,
        override_usage = "cargo llvm-lines [OPTIONS] -- [RUSTC OPTIONS]",
        disable_help_flag = true,
        disable_version_flag = true,
    )]
    LlvmLines {
        /// Set column by which to sort output table.
        #[arg(
            short,
            long,
            value_enum,
            value_name = "ORDER",
            default_value_t = SortOrder::Lines,
        )]
        sort: SortOrder,

        /// Display only functions matching the given regex.
        #[arg(long, value_name = "REGEX")]
        filter: Option<Regex>,

        /// Analyze existing .ll files that were produced by e.g.
        /// `RUSTFLAGS="--emit=llvm-ir" ./x.py build --stage 0 compiler/rustc`.
        #[arg(short, long, value_name = "FILES")]
        files: Vec<PathBuf>,

        // Run in a different mode that just filters some Cargo output and does
        // nothing else.
        #[arg(long, hide = true)]
        filter_cargo: bool,

        // All these options are passed through to the cargo rustc invocation.
        #[arg(short, long)]
        quiet: bool,
        #[arg(short, long, value_name = "SPEC")]
        package: Option<String>,
        #[arg(long)]
        lib: bool,
        #[arg(long, value_name = "NAME")]
        bin: Option<String>,
        #[arg(long, value_name = "NAME")]
        example: Option<String>,
        #[arg(long, value_name = "NAME")]
        test: Option<String>,
        #[arg(long)]
        release: bool,
        #[arg(long, value_name = "PROFILE-NAME")]
        profile: Option<String>,
        #[arg(long, value_name = "FEATURES")]
        features: Option<String>,
        #[arg(long)]
        all_features: bool,
        #[arg(long)]
        no_default_features: bool,
        #[arg(long, value_name = "TRIPLE")]
        target: Option<String>,
        #[arg(long, value_name = "PATH")]
        manifest_path: Option<String>,

        #[arg(short, long)]
        help: bool,
        #[arg(short = 'V', long)]
        version: bool,

        // Any additional flags for rustc taken after `--`.
        #[arg(last = true, hide = true)]
        rest: Vec<OsString>,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum SortOrder {
    Lines,
    Copies,
}
