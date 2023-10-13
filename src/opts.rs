use clap::{Parser, ValueEnum};
use regex::Regex;
use std::ffi::OsString;
use std::path::PathBuf;

const ABOUT: &str = "
Print the number of lines of LLVM IR that is generated for the current project.

Options shown below without an explanation mean the same thing as the
corresponding option of `cargo build`.";

const TEMPLATE: &str = "\
cargo-llvm-lines {version}
{author}
{about}

{usage-heading}
    {usage}

{all-args}";

// Help headings
const PACKAGE_SELECTION: &str = "Package Selection";
const TARGET_SELECTION: &str = "Target Selection";
const FEATURE_SELECTION: &str = "Feature Selection";
const COMPILATION_OPTIONS: &str = "Compilation Options";
const MANIFEST_OPTIONS: &str = "Manifest Options";

#[derive(Parser, Debug)]
#[command(
    name = "cargo-llvm-lines",
    bin_name = "cargo",
    author,
    version,
    disable_help_subcommand = true
)]
#[allow(dead_code)]
pub enum Subcommand {
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
    LlvmLines(LlvmLines),
}

#[derive(Parser, Debug)]
pub struct LlvmLines {
    /// Set column by which to sort output table.
    #[arg(
        short,
        long,
        value_enum,
        value_name = "ORDER",
        default_value_t = SortOrder::Lines,
    )]
    pub sort: SortOrder,

    /// Display only functions matching the given regex.
    #[arg(long, value_name = "REGEX")]
    pub filter: Option<Regex>,

    /// Analyze existing .ll files that were produced by e.g.
    /// `RUSTFLAGS="--emit=llvm-ir" ./x.py build --stage 0 compiler/rustc`.
    #[arg(short, long, value_name = "FILES")]
    pub files: Vec<PathBuf>,

    // The following options are passed through to the cargo rustc invocation.
    #[arg(short, long)]
    pub quiet: bool,
    #[arg(long, value_name = "WHEN", hide_possible_values = true)]
    pub color: Option<Coloring>,
    #[arg(long, value_name = "KEY=VALUE")]
    pub config: Vec<String>,
    #[arg(short = 'Z', value_name = "FLAG")]
    pub nightly_only_flags: Vec<String>,
    #[arg(short, long)]
    pub help: bool,
    #[arg(short = 'V', long)]
    pub version: bool,

    // Package selection
    #[arg(short, long, value_name = "SPEC", help_heading = PACKAGE_SELECTION)]
    pub package: Option<String>,

    // Target selection
    #[arg(long, help_heading = TARGET_SELECTION)]
    pub lib: bool,
    #[arg(long, value_name = "NAME", help_heading = TARGET_SELECTION)]
    pub bin: Option<String>,
    #[arg(long, value_name = "NAME", help_heading = TARGET_SELECTION)]
    pub example: Option<String>,
    #[arg(long, value_name = "NAME", help_heading = TARGET_SELECTION)]
    pub test: Option<String>,
    #[arg(long, value_name = "NAME", help_heading = TARGET_SELECTION)]
    pub bench: Option<String>,

    // Feature selection
    #[arg(short = 'F', long, value_name = "FEATURES", help_heading = FEATURE_SELECTION)]
    pub features: Option<String>,
    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub all_features: bool,
    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub no_default_features: bool,

    // Compilation options
    #[arg(short, long, value_name = "N", help_heading = COMPILATION_OPTIONS)]
    pub jobs: Option<usize>,
    #[arg(long, help_heading = COMPILATION_OPTIONS)]
    pub release: bool,
    #[arg(long, value_name = "PROFILE-NAME", help_heading = COMPILATION_OPTIONS)]
    pub profile: Option<String>,
    #[arg(long, value_name = "TRIPLE", help_heading = COMPILATION_OPTIONS)]
    pub target: Option<String>,
    #[arg(long, value_name = "DIRECTORY", help_heading = COMPILATION_OPTIONS)]
    pub target_dir: Option<PathBuf>,

    // Manifest options
    #[arg(long, value_name = "PATH", help_heading = MANIFEST_OPTIONS)]
    pub manifest_path: Option<PathBuf>,
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub frozen: bool,
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub locked: bool,
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub offline: bool,

    // Any additional flags for rustc taken after `--`.
    #[arg(last = true, hide = true)]
    pub rest: Vec<OsString>,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum SortOrder {
    Lines,
    Copies,
    Name,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}

#[test]
fn test_cli() {
    <Subcommand as clap::CommandFactory>::command().debug_assert();
}
