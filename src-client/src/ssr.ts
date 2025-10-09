import { registerEnv } from "mini-van-plate/shared";
import van from "mini-van-plate/van-plate";

const __dirname = import.meta.dirname;

registerEnv({ van });
const { h1, head, link, meta, script } = van.tags;

export async function serverResponse(req: Request): Promise<Response> {
	const url = new URL(req.url);

	if (url.pathname === "/favicon.ico") return new Response();

	if (url.pathname.endsWith(".js") || url.pathname.endsWith(".css"))
		return new Response(Bun.file("dist" + url.pathname));
	else if (url.pathname.match(/\.\S+$/))
		return new Response(Bun.file(__dirname + url.pathname));

	// biome-ignore lint: lazy
	let Page: any;
	const routePath = url.pathname === "/" ? "/index" : url.pathname;
	try {
		const pageModule = await import(`./routes${routePath}.ts`);

		Page = pageModule.default
			? pageModule.default()
			: h1("Page has no default export");
	} catch (err) {
		console.warn(`Page not found for ${url.pathname}`, err);
		Page = h1("404 Not Found");
	}

	const html = van.html(
		head(
			meta({ charset: "UTF-8" }),
			meta({
				name: "viewport",
				content: "width=device-width, initial-scale=1",
			}),

			link({ rel: "stylesheet", href: "/style.css" }),
			script({ type: "module", src: `${routePath}.js` }),
		),
		Page,
	);

	return new Response(html, {
		headers: { "Content-Type": "text/html; charset=UTF-8" },
	});
}

if (import.meta.main) {
	const server = Bun.serve({
		port: Bun.argv[2] ?? 5173,
		fetch: serverResponse,
	});
	console.log(`Server running at http://localhost:${server.port}`);
}
