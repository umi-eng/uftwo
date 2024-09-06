# `uftwo`

[![Crate](https://img.shields.io/crates/v/uftwo.svg)](https://crates.io/crates/uftwo)
[![Docs](https://docs.rs/uftwo/badge.svg)](https://docs.rs/uftwo)

For working with the [UF2 file format](https://github.com/microsoft/uf2).

Why the name? `uf2` was already taken and appears to be abandoned.

Warning: whilst this library is `0.1.x` there will be no SemVer compatibility guarantees. Use at your own risk.

## Using the library

```shell
cargo add uftwo
```

## Using the CLI

```shell
cargo install uftwo --features="cli"
```

## Features

- `defmt-03` enable [defmt](https://github.com/knurling-rs/defmt) `Format` on relevant types.
