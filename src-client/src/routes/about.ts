import { onPageLoad } from "../components/utils/hydrate";
import { env } from "mini-van-plate/shared";

export default function Page() {
	const { h1 } = env.van.tags;

	return h1("Hello from server!");
}

onPageLoad(Page);
