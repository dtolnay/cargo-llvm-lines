# cargo-llvm-lines

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/cargo--llvm--lines-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/cargo-llvm-lines)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cargo-llvm-lines.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cargo-llvm-lines)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/dtolnay/cargo-llvm-lines/CI/master?style=for-the-badge" height="20">](https://github.com/dtolnay/cargo-llvm-lines/actions?query=branch%3Amaster)

Count the number of lines of LLVM IR across all instantiations of a generic
function. Based on a suggestion from **@eddyb** on how to count monomorphized
functions in order to debug compiler memory usage, executable size and compile
time.

> **\<eddyb>** unoptimized LLVM IR<br>
> **\<eddyb>** first used grep '^define' to get only the lines defining function bodies<br>
> **\<eddyb>** then regex replace in my editor to remove everything before @ and everything after (<br>
> **\<eddyb>** then sort | uniq -c<br>

## Installation

Install with `cargo install cargo-llvm-lines`.

## Output

One line per function with three columns of output:

1. Total number of lines of LLVM IR generated across all instantiations of the
   function (and the percentage of the total).
2. Number of instantiations of the function (and the percentage of the total).
   For a generic function, roughly the number of distinct combinations of
   generic type parameters it is called with.
3. Name of the function.

```
$ cargo llvm-lines | head -20

  Lines         Copies       Function name
  -----         ------       -------------
  30737 (100%)  1107 (100%)  (TOTAL)
   1395 (4.5%)    83 (7.5%)  core::ptr::drop_in_place
    760 (2.5%)     2 (0.2%)  alloc::slice::merge_sort
    734 (2.4%)     2 (0.2%)  alloc::raw_vec::RawVec<T,A>::reserve_internal
    666 (2.2%)     1 (0.1%)  cargo_llvm_lines::count_lines
    490 (1.6%)     1 (0.1%)  <std::process::Command as cargo_llvm_lines::PipeTo>::pipe_to
    476 (1.5%)     6 (0.5%)  core::result::Result<T,E>::map
    440 (1.4%)     1 (0.1%)  cargo_llvm_lines::read_llvm_ir
    422 (1.4%)     2 (0.2%)  alloc::slice::merge
    399 (1.3%)     4 (0.4%)  alloc::vec::Vec<T>::extend_desugared
    388 (1.3%)     2 (0.2%)  alloc::slice::insert_head
    366 (1.2%)     5 (0.5%)  core::option::Option<T>::map
    304 (1.0%)     6 (0.5%)  alloc::alloc::box_free
    296 (1.0%)     4 (0.4%)  core::result::Result<T,E>::map_err
    295 (1.0%)     1 (0.1%)  cargo_llvm_lines::wrap_args
    291 (0.9%)     1 (0.1%)  core::char::methods::<impl char>::encode_utf8
    286 (0.9%)     1 (0.1%)  cargo_llvm_lines::run_cargo_rustc
    284 (0.9%)     4 (0.4%)  core::option::Option<T>::ok_or_else
```

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
