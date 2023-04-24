# cargo-llvm-lines

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/cargo--llvm--lines-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/cargo-llvm-lines)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cargo-llvm-lines.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cargo-llvm-lines)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/cargo-llvm-lines/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/cargo-llvm-lines/actions?query=branch%3Amaster)

Generic functions in Rust can be instantiated many times, which can increase
compile speed and memory use, and the size of compiled executables. This tool
measures the number and size of instantiations, indicating which parts of your
code could be rewritten to improve things.

## Installation

Install with `cargo install cargo-llvm-lines`.

## Output

Example output from running `cargo llvm-lines` on its own codebase:

```console
$ cargo llvm-lines | head -20

  Lines                Copies              Function name
  -----                ------              -------------
  52262                1872                (TOTAL)
   1815 (3.5%,  3.5%)     1 (0.1%,  0.1%)  <cargo_llvm_lines::Opt as clap::derive::Subcommand>::augment_subcommands
   1806 (3.5%,  6.9%)     1 (0.1%,  0.1%)  <cargo_llvm_lines::Opt as clap::derive::FromArgMatches>::from_arg_matches_mut
   1546 (3.0%,  9.9%)    34 (1.8%,  1.9%)  core::option::Option<T>::map
   1045 (2.0%, 11.9%)     5 (0.3%,  2.2%)  clap::parser::matches::arg_matches::ArgMatches::try_remove_arg_t
    738 (1.4%, 13.3%)     2 (0.1%,  2.3%)  alloc::slice::merge_sort
    648 (1.2%, 14.5%)     4 (0.2%,  2.5%)  alloc::raw_vec::RawVec<T,A>::grow_amortized
    645 (1.2%, 15.8%)     6 (0.3%,  2.8%)  <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
    636 (1.2%, 17.0%)    43 (2.3%,  5.1%)  <cargo_llvm_lines::Opt as clap::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}
    587 (1.1%, 18.1%)    15 (0.8%,  5.9%)  <core::result::Result<T,E> as core::ops::try_trait::Try>::branch
    565 (1.1%, 19.2%)     6 (0.3%,  6.2%)  core::iter::traits::iterator::Iterator::try_fold
    533 (1.0%, 20.2%)     1 (0.1%,  6.3%)  cargo_llvm_lines::print_table
    520 (1.0%, 21.2%)     6 (0.3%,  6.6%)  alloc::vec::Vec<T,A>::extend_desugared
    509 (1.0%, 22.2%)     5 (0.3%,  6.9%)  clap::parser::matches::any_value::AnyValue::downcast_into
    504 (1.0%, 23.1%)     5 (0.3%,  7.2%)  alloc::sync::Arc<T>::try_unwrap
    470 (0.9%, 24.0%)    11 (0.6%,  7.7%)  core::option::Option<T>::ok_or_else
    438 (0.8%, 24.9%)     2 (0.1%,  7.9%)  alloc::slice::merge
    414 (0.8%, 25.7%)     9 (0.5%,  8.3%)  core::result::Result<T,E>::and_then
```

There is one line per function with three columns of output:

1. Total number of lines of LLVM IR generated across all instantiations of the
   function (plus the percentage of the total and the cumulative percentage
   of all functions so far).
2. Number of instantiations of the function (plus the percentage of the total
   and the cumulative percentage of all functions so far). For a generic
   function, the number of instantiations is roughly the number of distinct
   combinations of generic type parameters it is called with.
3. Name of the function.

## Multicrate Projects

Interpreting the output in the presence of multiple crates and generics can be
tricky. `cargo llvm-lines` only shows the contribution of the root crate;
dependencies are not included. To assess the contribution of an intermediate
crate, use the `-p` flag:

```console
$ cargo llvm-lines -p some-depenency
```

Note however, that Rust generics are monomorphised &mdash; a generic function
will be accounted for in the crates that use it, rather than in the defining
crate.

There is a trick to get a holistic view: enabling link time optimization causes
all code generation to happen in the root crate. So you can use the following
invocation to get a full picture:

```console
$ CARGO_PROFILE_RELEASE_LTO=fat cargo llvm-lines --release
```

## Genesis

Based on a suggestion from **@eddyb** on how to count monomorphized functions
in order to debug compiler memory usage, executable size and compile time.

> **\<eddyb>** unoptimized LLVM IR<br>
> **\<eddyb>** first used grep '^define' to get only the lines defining function bodies<br>
> **\<eddyb>** then regex replace in my editor to remove everything before @ and everything after (<br>
> **\<eddyb>** then sort | uniq -c<br>

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
