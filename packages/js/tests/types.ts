import { TextDecoder, TextEncoder } from "../src/text.ts";

// Check constructor compatibility in addition to the instance implementations.
TextDecoder satisfies typeof globalThis.TextDecoder;
TextEncoder satisfies typeof globalThis.TextEncoder;

// Browser and Node capabilities must not leak into the evaluator's environment.
// @ts-expect-error The evaluator does not expose a document.
void document;
// @ts-expect-error The evaluator does not expose fetch.
void fetch;
// @ts-expect-error The evaluator does not expose Node's process.
void process;
// @ts-expect-error The evaluator does not expose timers.
void setTimeout;
