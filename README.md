bit-long-vec
============
[![crates.io](https://img.shields.io/crates/v/bit-long-vec.svg)](https://crates.io/crates/bit-long-vec)
[![Build Status](https://travis-ci.com/eihwaz/bit-long-vec.svg?branch=master)](https://travis-ci.com/eihwaz/bit-long-vec)
[![codecov](https://codecov.io/gh/eihwaz/bit-long-vec/branch/master/graph/badge.svg)](https://codecov.io/gh/eihwaz/bit-long-vec)

Vector with fixed bit sized values stored in long. Effective to reduce the amount of memory needed for storage
values whose size is not a power of 2. As drawback to set and get values uses additional CPU cycles for bit operations.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
bit-long-vec = "0.2"
```

## Example

In this particular scenario, we want to store 10 bit values. It takes 200 bytes to store 100 values using short.
To store 100 values using a bit long vector, 15 lengths are required, which is 120 bytes. (**-40%**). 

```rust
use bit_long_vec::BitLongVec;

let mut vec = BitLongVec::with_fixed_capacity(100, 10);

for index in 0..100 {
    vec.set(index, 1023);

    assert_eq!(vec.get(index), 1023);
}
```