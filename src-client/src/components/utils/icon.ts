import { icons } from "@iconify-json/mingcute";

export function icon(str: string): string | undefined {
	if (process.env.BUILD_TARGET === "browser") return;

	const [collection, name] = str.split(":") ?? [];
	if (!collection || !name) return;

	return icons.icons?.[name].body;
}
