# Provisioned ESM packages

Put `tangram.packages.json` next to your Tangram root module to opt that directory into package resolution:

```json
{"conditions": ["tangram"]}
```

Keep a `package.json` there declaring the packages used by the root module, and provision those packages and their dependencies under its `node_modules` directory before invoking Tangram. Use an existing package manager with a frozen lockfile, or copy an already provisioned installation. Keep each package's original `package.json` and ESM files. Symlinks must resolve within this directory; installations that link to an external store must be materialized within the scope first.

For example, with an existing ESM package named `example`:

```json
{"type":"module","dependencies":{"example":"1.0.0"}}
```

```typescript
// tangram.ts
import { value } from "example";
import { helper } from "example/helpers";
export default () => helper(value);
```

Run `tg build .`, or capture with `tg checkin .` and build the resulting artifact. Tangram does not install packages or interpret dependency version ranges. The provisioner selects versions; the captured files establish their content identity. Nested `node_modules` installations provide ordinary per-package dependency scopes. An installation owns its dependency declarations independently of nested manifests that select module interpretation. Each package must declare cross-package imports in `dependencies`, `optionalDependencies`, or `peerDependencies`; undeclared dependencies in package code are errors.

## Architecture and future integration

Tangram owns supported ECMAScript module resolution, dependency capture, and
execution against the captured graph. Ecosystem bridges should be separate
packages or recipes that interpret lockfiles, acquire verified package artifacts,
establish exact dependency scopes, and model any required package builds.
Package acquisition, module resolution, and evaluator capabilities are separate
responsibilities. Making a library available does not grant it additional host
APIs; libraries requiring platform services need compatible implementations.

External provisioning is the transitional workflow. Resolution of `package.json`
imports and exports is an intended continuing responsibility, even when a bridge
eventually provisions packages through Tangram. Resolving module imports during
capture and loading exact recorded edges during execution preserves that
separation. A bridge should not need an independently maintained module resolver
or a library-specific module registry. Effect is a prospective consumer of this
general integration, not a special case in the engine.

Keep enablement and resolver conditions in the separate
`tangram.packages.json` file. This makes the package scope explicit and lets the
consumer compose Tangram configuration with unchanged upstream manifests.
Packages describe their conditional exports; the consuming scope selects the
runtime conditions. Embedding equivalent settings in `package.json` would save
one file but blur that ownership boundary and require rules distinguishing root
settings from dependency settings. Supporting both forms would additionally
require precedence and conflict rules; only the separate file is supported.

The existing ESM capture, loading, type-resolution, editor, and checkout work
remains relevant to future bridges. Lockfile interpretation, registry acquisition,
and package installation policy should stay outside the engine. Further core
changes should address demonstrated integration gaps rather than introduce
ecosystem-specific provisioning.

Direct composition from Tangram-produced package artifacts into package capture
is an open integration question, not a verified workflow here. A future bridge
should exercise that path and determine whether an unnecessary host checkout
round trip can be avoided before expanding the input interface. Lifecycle
scripts, generated files, native addons, and downloaded binaries also require
explicit build modeling; a lockfile alone does not describe their build graph.

## Resolution and capture

Runtime resolution supports `.mjs` and `.js` in packages whose metadata says `"type":"module"`. It supports package `exports` and `imports`, subpaths, patterns, conditional targets, self references, relative imports with explicit extensions, and literal `import("specifier")` expressions. The `import` condition is always active, alongside the explicitly listed conditions. Package-map property order determines precedence; `default` retains its package-map meaning. `types` and `require` cannot be configured as runtime conditions.

The compiler separately enables `types` and recognizes `.d.ts` and `.d.mts`, declaration extension aliases (`.ts` to `.d.ts`, `.mts` to `.d.mts`, and the corresponding `.js`/`.mjs` spellings), import-type expressions, and `types`/`typings` entry fields. Editor sessions revalidate package targets and track changes to external declaration files. Capture records separate runtime and type edges. Declaration targets are never selected by runtime package resolution. Existing declaration paths retain their identity. These substitutions do not enable execution of TypeScript package sources or CommonJS; `.cts`, `.cjs`, and `.d.cts` package targets remain unsupported.

Check-in parses reachable modules and follows their imports and re-exports automatically, including cycles. Dependencies explicitly reached this way are captured even when `node_modules` is ignored during directory walking. The graph includes the scope configuration and resolution metadata. Checkout records exact package and metadata targets in the existing dependency xattrs, so recapture does not depend on their former filesystem locations. Stored execution follows exact captured edges, preserving existing authorization and module-identity handling; it performs no package search or registry lookup. Filesystem and editor resolution use the same scoped resolver. Watches conservatively invalidate a package scope on changes, including newly created metadata and resolution candidates.

Existing Tangram object references, explicit import attributes, and native Tangram module imports retain their semantics. In a native Tangram module, a bare name not declared as a package remains a Tangram reference. Package code cannot fall back to Tangram's registry for an undeclared bare dependency. A nested Tangram root without its own scope marker ends package-scope discovery.

## Limits

This supplies module resolution, not a Node.js runtime. CommonJS execution, Node built-ins, native addons, JSON modules, network module URLs, package lifecycle scripts, and unrestricted host APIs are not provided. JavaScript globals and capabilities remain those of the selected Tangram evaluator. Packages that need additional Node or browser APIs may resolve successfully but cannot necessarily execute.

Computed dynamic-import targets are rejected during package-module analysis, even when their value happens to name an already captured module. Dynamic import attributes must use the existing literal attribute form. Runtime relative imports and `main` entries require explicit file extensions; legacy extension and implicit directory-index guessing are unsupported; query-string and fragment identities are unsupported. Type declarations must also be resolvable from the provisioned scope.

The pinned QuickJS engine crashes on some cycles through imported bindings that are re-exported. The same crash occurs with native `.tg.ts` modules without package resolution. V8 supports the tested cycle; use V8 for that graph shape. Acyclic package graphs were exercised through both engines.

## Implementation choice

The shared Rust package-resolution boundary uses [Oxc Resolver](https://github.com/oxc-project/oxc-resolver), pinned in Cargo.lock. The workspace requires Rust 1.96, matching the existing pinned Oxc parser; the resolver requires Rust 1.95. Its maintained package-map implementation and custom filesystem interface avoid duplicating exports/imports precedence and target validation. Tangram owns the explicit scope, dependency declarations, supported module kinds, runtime/type separation, and graph capture. Every resolver filesystem operation is confined to the canonical scope, including symlink targets. A fresh resolver avoids stale internal filesystem caches.

The main alternative was a local implementation of package-map semantics. It would reduce the dependency footprint but require Tangram to maintain conditional maps, pattern matching, target rejection, and their conformance cases. The runtime's existing module callbacks do not supply those semantics. Oxc's support for CommonJS resolution does not enable CommonJS execution here. No JavaScript dependencies are bundled into the evaluator's main module, and there is no public resolver-plugin framework.

After validation, a downstream source consumer can pin the resulting Tangram revision, update its source provenance, and run its own bootstrap and self-build checks. This change does not update Universe or establish its self-build provenance.
