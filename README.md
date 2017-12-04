# cargo-llvm-lines

[![Build Status](https://travis-ci.org/dtolnay/cargo-llvm-lines.svg?branch=master)](https://travis-ci.org/dtolnay/cargo-llvm-lines)
[![Latest Version](https://img.shields.io/crates/v/cargo-llvm-lines.svg)](https://crates.io/crates/cargo-llvm-lines)

Count the number of lines of LLVM IR across all instantiations of a generic
function. Based on a suggestion from **@eddyb** on how to a count monomorphized
functions in order to debug compiler memory usage and executable size.

> **\<eddyb>** unoptimized LLVM IR<br>
> **\<eddyb>** first used grep '^define' to get only the lines defining function bodies<br>
> **\<eddyb>** then regex replace in my editor to remove everything before @ and everything after (<br>
> **\<eddyb>** then sort | uniq -c<br>

## Installation

Install with `cargo install cargo-llvm-lines`.

## Output

One line per function with three columns of output:

1. Total number of lines of LLVM IR generated across all instantiations of the
   function.
2. Number of instantiations of the function. For a generic function, roughly the
   number of distinct combinations of generic type parameters it is called with.
3. Name of the function.

```
$ cargo llvm-lines | head -20

   2447  130  core::ptr::drop_in_place
   1720   19  <core::option::Option<T>>::map
    862    2  core::str::pattern::TwoWaySearcher::next
    726    4  <alloc::raw_vec::RawVec<T, A>>::double
    698    4  <alloc::raw_vec::RawVec<T, A>>::reserve
    677    6  <core::result::Result<T, E>>::map
    602    1  cargo_llvm_lines::read_llvm_ir
    598    5  <alloc::vec::Vec<T>>::extend_desugared
    477    1  cargo_llvm_lines::count_lines
    476    9  <alloc::raw_vec::RawVec<T, A>>::dealloc_buffer
    464   10  alloc::heap::box_free
    452    5  <alloc::vec::Vec<T> as alloc::vec::SpecExtend<T, I>>::spec_extend
    448    1  alloc::slice::merge_sort
    436    1  <std::process::Command as cargo_llvm_lines::PipeTo>::pipe_to
    419    4  <core::slice::Iter<'a, T> as core::iter::iterator::Iterator>::next
    400    1  <core::hash::sip::Sip13Rounds as core::hash::sip::Sip>::d_rounds
    378    9  <alloc::raw_vec::RawVec<T, A>>::current_layout
    362    3  <alloc::raw_vec::RawVec<T, A>>::allocate_in
    354    4  <alloc::vec::Vec<T>>::push
    341    4  <[T] as core::slice::SliceExt>::iter
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in cargo-llvm-lines by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
