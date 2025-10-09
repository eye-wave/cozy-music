import Player from "../components/Player";
import { onPageLoad } from "../components/utils/hydrate";
import { env } from "mini-van-plate/shared";

export default function Page() {
	const { div, h1 } = env.van.tags;

	return div(
		h1({ class: "text-3xl font-sans text-rose-500" }, "Hello world!"),
		Player(),
	);
}

onPageLoad(Page);
