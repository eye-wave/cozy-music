import { serverResponse } from "./src/ssr";
import * as Bun from "bun";
import { load } from "cheerio";
import { readdirSync, statSync } from "fs";
import { join, extname, basename } from "path";
import { optimize, type Config } from "svgo";
import { minify, type MinifyOptions } from "terser";

const { $ } = Bun;

// 1. Clean & copy assets
await $`rm -rf dist`;
await $`mkdir -p dist/assets`;
await $`cp src/assets/ -r dist`;

// 2. Tailwind & client build
await $`bun run build:tailwind`;
await $`bun run build:client`;

// 3. Helper: recursively find all page routes
function findPages(dir: string, prefix = ""): string[] {
	const entries = readdirSync(dir);
	let pages: string[] = [];

	for (const entry of entries) {
		const fullPath = join(dir, entry);
		const stat = statSync(fullPath);

		if (stat.isDirectory()) {
			pages = pages.concat(findPages(fullPath, `${prefix}/${entry}`));
		} else if (extname(entry) === ".ts" || extname(entry) === ".js") {
			const name =
				entry === "index.ts" || entry === "index.js"
					? ""
					: basename(entry, extname(entry));
			pages.push(`${prefix}/${name}`);
		}
	}
	return pages;
}

const pageRoutes = findPages("./src/routes").map((p) => (p === "" ? "/" : p));

// 4. Pre-render all pages
for (const route of pageRoutes) {
	let html = await serverResponse(new Request(`http://localhost${route}`)).then(
		(res) => res.text(),
	);

	// 4a. Optimize inline SVGs
	const svgOptions: Config = { floatPrecision: 1 };
	const $page = load(html);
	$page("svg").each((_, el) => {
		const svg = $page.html(el);
		const res = optimize(svg, svgOptions);
		if (res.data.length < svg.length) $page(el).replaceWith(res.data);
	});
	html = $page.html();

	// 4b. Inline CSS & JS if --inline
	const isInline = process.argv.includes("--inline");
	if (isInline) {
		$page('link[rel="icon"], link[rel="shortcut icon"]').remove();

		await Promise.all(
			$page('link[rel="stylesheet"]')
				.map(async (_, el) => {
					const href = $page(el).attr("href");
					if (!href) return;
					const css = await Bun.file("dist" + href).text();
					$page(el).replaceWith(`<style>${css}</style>`);
				})
				.get(),
		);

		await Promise.all(
			$page("script[src]")
				.map(async (_, el) => {
					const src = $page(el).attr("src");
					if (!src) return;
					const js = await Bun.file("dist" + src).text();
					$page(el).replaceWith(`<script type="module">${js}</script>`);
				})
				.get(),
		);

		await Promise.all(
			$page('img[src$=".svg"]')
				.map(async (_, el) => {
					const src = $page(el).attr("src");
					if (!src) return;
					const svg = await Bun.file("dist" + src).text();
					$page(el).replaceWith(svg);
				})
				.get(),
		);

		html = $page.html();
	}

	// 4c. Write HTML to dist
	const outFile = route === "/" ? "dist/index.html" : `dist${route}.html`;
	await $`mkdir -p ${join(outFile, "..")}`;
	await Bun.write(outFile, html);
}

// 5. Minify all JS in dist
const jsFiles: string[] = [];
function findJsFiles(dir: string) {
	for (const f of readdirSync(dir)) {
		const fullPath = join(dir, f);
		if (statSync(fullPath).isDirectory()) findJsFiles(fullPath);
		else if (extname(f) === ".js") jsFiles.push(fullPath);
	}
}
findJsFiles("dist");

const MINIFY_OPTIONS: MinifyOptions = {
	compress: {
		evaluate: true,
		unused: true,
		dead_code: true,
		passes: 3,
		module: true,
		ecma: 2020,
	},
	mangle: true,
	format: { comments: false },
};

for (const file of jsFiles) {
	const code = await Bun.file(file).text();
	const mini = await minify(code, MINIFY_OPTIONS);
	if (mini.code) await Bun.write(file, mini.code);
}
