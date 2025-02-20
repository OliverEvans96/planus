Planus – alternative flatbuffer implementation
===============================================

[![Build Status](https://github.com/planus-org/planus/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/planus-org/planus/actions/workflows/rust.yml)
![Rustc Version 1.57+](https://img.shields.io/badge/rustc-1.57+-lightgray.svg)
[![License](https://img.shields.io/crates/l/planus)](https://crates.io/crates/planus)
[![Crates.io](https://img.shields.io/crates/v/planus)](https://crates.io/crates/planus)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/planus)


Planus is an alternative compiler for [flatbuffers](https://google.github.io/flatbuffers/), an efficient cross platform serialization library.

Goals
-----

* **User experience**: Our command-line interface has excellent error and help messages. We aim to output good error messages even for non-supported features.
* **Ideomatic code**: We want to generate highly idiomatic code that should be very familiar to programmers of the target programming language.
* **Safety**: Any undefined/unsafe behavior in the generated code is considered a critical bug.
* **Performance**: We want to be at least as performant as the official implementation.
* **Modularity**: We have written our code such that parsing, validation and translation are clearly separated. We hope this will make it easy to implement additional backends with full support.
* **Opinionated**: We are in some cases more strict than the official implementation.
* **Developer tools**: We want to build good developer tools using our library. This includes at least a schema formatter and a tool to output a [DOT graph](https://en.wikipedia.org/wiki/DOT_(graph_description_language)) of the types in a schema.
* **Rust**: By choosing to use Rust for our compiler, we are able to utilize excellent crates such as [clap](https://github.com/clap-rs/clap), [LALRPOP](https://github.com/lalrpop/lalrpop) and [codespan](https://github.com/brendanzab/codespan) as force multipliers.

Non-goals
---------

* **Full feature parity**: Certain features are difficult to re-implement in a clean fashion.
* **API-level compatibility**: We aim for binary-level compatibility, but the code we generate will not works directly with the APIs of the official implementation.
* **Language-specific extensions**: We do not plan to support any extensions that break support between different languages.
* **Validation-free access**: Validation-free access makes it very easy to shoot yourself in the foot. We will not provide such APIs.

Languages supported
-------------------

Currently we only support Rust, though we plan to add support for more languages in the future. Pull requests are welcome!

Getting started
---------------

For now our documention could use a bit of improvement, but we do have an example to help you get started. Look in [examples/rust](examples/rust).


Status of the implementation
----------------------------

We support most of the base language, though some parts have not been tested in-depth.

Things we do not currently support:

* `rpc_service`
* `file_extension`, `file_identifier` and `root_type`
* Fixed-size arrays
* Vectors of unions
* Any attribute besides `required`, `deprecated`, `id` or `force_align`.
* Some of the more exotic literal values, like hexadecimal floats or unicode surrogate pair parsing.
* JSON conversion.

Things we will probably never support:

* `native_includes`
* More than one `namespace` per file.
* Flexbuffers.


I think I found a bug!
----------------------

Please file an issue! We also consider poor error messages or unintuitive behavior to be high-priority bugs.
