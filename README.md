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
  51637                1222                (TOTAL)
   2240 (4.3%,  4.3%)     1 (0.1%,  0.1%)  <cargo_llvm_lines::opts::LlvmLines as clap_builder::derive::Args>::augment_args
   1190 (2.3%,  6.6%)     1 (0.1%,  0.2%)  <cargo_llvm_lines::opts::LlvmLines as clap_builder::derive::FromArgMatches>::from_arg_matches_mut
   1005 (1.9%,  8.6%)     3 (0.2%,  0.4%)  alloc::raw_vec::RawVec<T,A>::grow_amortized
    973 (1.9%, 10.5%)     7 (0.6%,  1.0%)  clap_builder::parser::matches::arg_matches::ArgMatches::try_remove_arg_t
    939 (1.8%, 12.3%)     7 (0.6%,  1.6%)  alloc::sync::Arc<T>::try_unwrap
    935 (1.8%, 14.1%)     6 (0.5%,  2.0%)  <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
    861 (1.7%, 15.8%)     7 (0.6%,  2.6%)  alloc::sync::Arc<dyn core::any::Any+core::marker::Send+core::marker::Sync>::downcast
    761 (1.5%, 17.2%)     5 (0.4%,  3.0%)  alloc::vec::Vec<T,A>::extend_desugared
    638 (1.2%, 18.5%)     1 (0.1%,  3.1%)  cargo_llvm_lines::table::print
    599 (1.2%, 19.6%)    16 (1.3%,  4.4%)  core::option::Option<T>::ok_or_else
    592 (1.1%, 20.8%)     2 (0.2%,  4.6%)  core::slice::sort::merge
    574 (1.1%, 21.9%)     2 (0.2%,  4.7%)  core::slice::sort::merge_sort
    561 (1.1%, 23.0%)     7 (0.6%,  5.3%)  clap_builder::parser::matches::any_value::AnyValue::downcast_into
    556 (1.1%, 24.1%)     4 (0.3%,  5.6%)  <core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next
    541 (1.0%, 25.1%)    16 (1.3%,  7.0%)  core::option::Option<T>::map
    536 (1.0%, 26.1%)     8 (0.7%,  7.6%)  <alloc::sync::Weak<T> as core::ops::drop::Drop>::drop
    533 (1.0%, 27.2%)     1 (0.1%,  7.7%)  core::str::pattern::simd_contains
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

## Acknowledgements

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
