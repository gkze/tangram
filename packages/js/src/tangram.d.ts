/// <reference lib="es2023" />

export * from "@tangramdotdev/client";
export as namespace Tangram;
export as namespace tg;

declare global {
	interface TextDecoderOptions {
		fatal?: boolean;
		ignoreBOM?: boolean;
	}

	interface TextDecodeOptions {
		stream?: boolean;
	}

	interface TextEncoderEncodeIntoResult {
		read: number;
		written: number;
	}

	interface TextEncoder {
		readonly encoding: string;
		encode(input?: string): Uint8Array<ArrayBuffer>;
		encodeInto(
			source: string,
			destination: Uint8Array,
		): TextEncoderEncodeIntoResult;
	}

	interface TextDecoder {
		readonly encoding: string;
		readonly fatal: boolean;
		readonly ignoreBOM: boolean;
		decode(
			input?: ArrayBufferLike | ArrayBufferView,
			options?: TextDecodeOptions,
		): string;
	}

	var TextEncoder: {
		prototype: TextEncoder;
		new (): TextEncoder;
	};

	var TextDecoder: {
		prototype: TextDecoder;
		new (label?: string, options?: TextDecoderOptions): TextDecoder;
	};

	interface ImportMeta {
		module: tg.Module;
	}

	interface Console {
		/** Write to stdout. */
		log(...args: Array<unknown>): void;

		/** Write to stderr. */
		error(...args: Array<unknown>): void;
	}

	var console: Console;

	function Tangram(
		strings: TemplateStringsArray,
		...placeholders: Tangram.Args<Tangram.Template.Arg>
	): Tangram.Template.Builder;
	function Tangram(
		...args: Tangram.Args<Tangram.Template.Arg>
	): Tangram.Template.Builder;

	function tg(
		strings: TemplateStringsArray,
		...placeholders: tg.Args<tg.Template.Arg>
	): tg.Template.Builder;
	function tg(...args: tg.Args<tg.Template.Arg>): tg.Template.Builder;
}
