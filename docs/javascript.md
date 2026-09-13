# JavaScript evaluation

V8 and QuickJS share the JavaScript initialization and public declarations in
`packages/js`. Capabilities are independent of how modules are provisioned.

## Text encoding

Both evaluators expose deterministic `TextEncoder` and `TextDecoder` globals.
`TextEncoder` encodes UTF-8, replaces unpaired UTF-16 surrogates, and supports
`encodeInto` with UTF-16 read counts and whole-scalar writes. `TextDecoder`
accepts buffer sources, supports encoding labels, replacement or fatal errors,
BOM handling, and incremental decoding with `stream`. Decoder state belongs to
the JavaScript object and is collected with it; no native handles or host
permissions are involved. `TextEncoderStream` and `TextDecoderStream` are not
provided.

The shared initialization bundles the MIT-licensed
[`@kayahr/text-encoding`](https://github.com/kayahr/text-encoding) implementation,
pinned in `bun.lock`. It supplies the encoding algorithms and tables, including
legacy decoders. Tangram's small adapter restricts the encoder to the standard
UTF-8 API and applies string, dictionary, and buffer argument validation. This
keeps engine behavior aligned without duplicating Unicode state machines.
Tangram's existing `tg.encoding.utf8` remains a separate strict conversion API.

The strongest alternative is `encoding_rs` behind native evaluator bindings.
It supplies maintained decoding algorithms, but integrating streaming decoder
ownership and JavaScript string/buffer conversions separately into V8 and
QuickJS adds lifecycle and boundary code. The pure JavaScript implementation
reuses the shared evaluator initialization and includes upstream malformed-input
and streaming regression tests.

The evaluator type-checks against ECMAScript and Tangram's public ambient
declarations. An internal declaration supplies only the dependency's
`BufferSource` alias. Instance and constructor conformance checks keep the
implementation aligned with the public API. Host-side tests use a separate
configuration with native runtime types; those types do not enter the evaluator.

Regression coverage includes malformed UTF-8, surrogate replacement, bounded
writes, buffer views, streaming boundaries, BOM behavior, fatal-error recovery,
and representative legacy encodings. This is not a claim that Tangram has
passed the entire WHATWG Encoding or Web IDL conformance suite.
