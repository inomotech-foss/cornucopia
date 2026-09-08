# Examples
The repository contains a few examples to get you going.

* The basic example showcases the basic workflow with simple queries. This example is available in both [sync](https://github.com/cornucopia-rs/cornucopia/tree/main/examples/basic_sync) and [async](https://github.com/cornucopia-rs/cornucopia/tree/main/examples/basic_async) versions.
* The [automatic query build](https://github.com/cornucopia-rs/cornucopia/tree/main/examples/auto_build) example showcases how to integrate Cornucopia's API inside a build script to automatically rebuild your Rust queries when your PostgreSQL queries change.
* The [custom types](https://github.com/cornucopia-rs/cornucopia/tree/main/examples/custom_types) example showcases how to specify custom Rust types and several other options in `cornucopia.toml`.
* The [shared runtime](https://github.com/cornucopia-rs/cornucopia/tree/main/examples/shared_runtime) example showcases how to move the client scaffold out of the generated crate and into a crate of its own, which matters when a project generates several crates.
