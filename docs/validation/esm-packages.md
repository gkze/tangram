# Provisioned package acceptance

This records integration evidence, not a library-specific runtime contract.
See [package resolution](../esm-packages.md) and
[evaluator capabilities](../javascript.md) for the general behavior.

## Effect acceptance

The opt-in CLI test `build/esm_effect.nu` takes an already prepared Effect
package directory through `TANGRAM_TEST_EFFECT_PACKAGE`. It copies the complete
package into its private scope, preserving JavaScript, declarations, and the
original manifest, and does not modify the provisioner's installation. With the
normal local backends and both evaluator features compiled:

```sh
cargo build --features quickjs
# On macOS, include the provisioned XZ runtime directory if the executable needs it.
export DYLD_LIBRARY_PATH="$TANGRAM_RUNTIME_LIBRARY_PATH"
export TANGRAM_TEST_EFFECT_PACKAGE=/absolute/path/to/prepared/effect
nu packages/cli/test.nu --no-cloud --tangram-path target/debug/tangram build/esm_effect
```

The test requires the default V8 evaluator for the supported acceptance result.
It exercises `effect/Effect`, pure Schema struct/number decoding, Effect
composition, Base64 encoding, and imports from the root `effect` barrel. Every
case runs directly and again from a captured artifact with its source directory
moved away and caching disabled. The test uses 1 GiB LMDB maps for this graph instead of the harness's 10 MiB small-fixture maps. The harness owns and cleans up its servers.
Without the environment variable, this test reports an explicit skip.

On macOS ARM64, the unmodified prepared Effect 4.0.0-rc.112 package passed these
operations with V8. Subpath and root-barrel success establishes this pure subset,
not every Effect operation or its platform integrations. No timers, networking,
filesystem access, process API, or Node compatibility is added by text encoding.
QuickJS's existing imported-binding cycle limitation still applies; do not infer
Effect compatibility from its passing encoding tests. Linux execution and the
complete Encoding/Web IDL conformance suites were not validated in this run.
