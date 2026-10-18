umut_clean() { :; }
umut_prettier() { printf 'export const   x  =  1\n' > src/lib/ugly.ts; }
umut_any() { printf 'export function f(v: any): unknown {\n\treturn v;\n}\n' > src/lib/a.ts; }
umut_ts_ignore() { printf '// @ts-ignore\nexport const n: number = "x";\n' > src/lib/a.ts; }
umut_ts_expect_nodesc() { printf '// @ts-expect-error\nexport const n: number = "x";\n' > src/lib/a.ts; }
umut_ts_expect_desc() { printf '// @ts-expect-error cas volontaire du test de typage\nexport const n: number = "x";\n' > src/lib/a.ts; }
umut_as_cast() { printf 'export const n = JSON.parse("1") as number;\n' > src/lib/a.ts; }
umut_console() { printf 'export function f(p: string): void {\n\tconsole.log(p);\n}\n' > src/lib/a.ts; }
umut_html() { printf '<script lang="ts">\n\tlet { h }: { h: string } = $props();\n</script>\n\n{@html h}\n' > src/lib/H.svelte; }
umut_complexity() { cat > src/lib/a.ts <<'X'
export function f(a: number, b: number): number {
	let n = 0;
	if (a > 1) n++;
	if (a > 2) n++;
	if (a > 3) n++;
	if (a > 4) n++;
	if (a > 5) n++;
	if (b > 1) n++;
	if (b > 2) n++;
	if (b > 3) n++;
	if (b > 4) n++;
	if (b > 5) n++;
	return n;
}
X
}
umut_depth() { cat > src/lib/a.ts <<'X'
export function f(a: number): number {
	if (a > 1) {
		if (a > 2) {
			if (a > 3) {
				if (a > 4) {
					return 1;
				}
			}
		}
	}
	return 0;
}
X
}
umut_params() { printf 'export function f(a: number, b: number, c: number, d: number): number {\n\treturn a + b + c + d;\n}\n' > src/lib/a.ts; }
umut_long_fn() { { echo 'export function f(): number {'; echo '	let n = 0;'; for i in $(seq 1 45); do echo "	n += $i;"; done; echo '	return n;'; echo '}'; } > src/lib/a.ts; }
umut_long_file() { { for i in $(seq 1 260); do echo "export const c$i = $i;"; done; } > src/lib/a.ts; }
SW='export type State = { kind: "idle" } | { kind: "running" } | { kind: "done" };
'
umut_switch_missing() { { echo "$SW"; cat <<'X'
export function label(s: State): string {
	switch (s.kind) {
		case 'idle':
			return 'a';
		case 'running':
			return 'b';
	}
	return '';
}
X
} > src/lib/a.ts; }
umut_switch_ok() { { echo "$SW"; cat <<'X'
function assertNever(x: never): never {
	throw new Error(String(x));
}

export function label(s: State): string {
	switch (s.kind) {
		case 'idle':
			return 'a';
		case 'running':
			return 'b';
		case 'done':
			return 'c';
		default:
			return assertNever(s);
	}
}
X
} > src/lib/a.ts; }
umut_switch_default_only() { { echo "$SW"; cat <<'X'
export function label(s: State): string {
	switch (s.kind) {
		case 'idle':
			return 'a';
		default:
			return 'z';
	}
}
X
} > src/lib/a.ts; }
umut_tauri_outside() { printf "declare module '@tauri-apps/api/core' {\n\texport function invoke(c: string): Promise<number>;\n}\n" > src/tauri-stub.d.ts; printf "import { invoke } from '@tauri-apps/api/core';\nexport const f = () => invoke('x');\n" > src/lib/a.ts; }
umut_tauri_in_api() { printf "declare module '@tauri-apps/api/core' {\n\texport function invoke(c: string): Promise<number>;\n}\n" > src/tauri-stub.d.ts; mkdir -p src/lib/api; printf "import { invoke } from '@tauri-apps/api/core';\nexport const f = () => invoke('x');\n" > src/lib/api/a.ts; }
umut_svelte_check_type() { printf 'export const n: number = "x";\n' > src/lib/a.ts; }
umut_svelte_check_a11y() { printf '<script lang="ts">\n</script>\n\n<img src="/x.png" />\n' > src/lib/Img.svelte; }
umut_svelte_check_prop() { printf '<script lang="ts">\n\timport Hello from "./Hello.svelte";\n</script>\n\n<Hello nom="x" />\n' > src/lib/Use.svelte; }
umut_test_fail() { sed -i "s/toBe('Bonjour Ana')/toBe('Salut Ana')/" src/lib/greet.test.ts; }
umut_test_no_assert() { printf "import { it } from 'vitest';\nimport { greet } from './greet';\n\nit('sans assertion', () => {\n\tgreet('x');\n});\n" > src/lib/b.test.ts; }
umut_test_skip() { sed -i "s/it('salue'/it.skip('salue'/" src/lib/greet.test.ts; }
umut_test_only() { sed -i "s/it('salue'/it.only('salue'/" src/lib/greet.test.ts; }
umut_color_hardcoded() { printf '<p>x</p>\n\n<style>\n\tp {\n\t\tcolor: #ff0000;\n\t}\n</style>\n' > src/lib/Col.svelte; }
