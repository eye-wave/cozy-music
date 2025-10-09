import { invoke } from "@tauri-apps/api/core";
import { env } from "mini-van-plate/shared";

export default function Player() {
	const { div, button } = env.van.tags;

	async function play() {
		await invoke("load_song", {
			path: "/home/eyewave/Music/soundboard/nega.mp3",
		});
	}

	return div(
		{ class: "w-full mx-2 h-32 bg-stone-600" },

		button({ onclick: play }, "Play"),
	);
}
