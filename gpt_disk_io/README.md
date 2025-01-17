# `gpt_disk_io`

[![Crates.io](https://img.shields.io/crates/v/gpt_disk_io)](https://crates.io/crates/gpt_disk_io) 
[![Docs.rs](https://docs.rs/gpt_disk_io/badge.svg)](https://docs.rs/gpt_disk_io)

`no_std` library for reading and writing [GPT] (GUID Partition Table)
disk data structures through a block IO interface.

See also the [`gpt_disk_types`] package.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_types`]: https://crates.io/crates/gpt_disk_types

## Features

* `std`: Enables the `StdBlockIo` type, as well as `std::error::Error`
  implementations for all of the error types. Off by default.
  
## Minimum Supported Rust Version (MSRV)

The current MSRV is 1.60.0 since that's the minimum for `uguid` and
`gpt_disk_types`. Feel free to file an issue or create a PR if you have
a use case that requires an older version.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
