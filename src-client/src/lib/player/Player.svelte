<script lang="ts">
	import { onMount } from "svelte";
	import { PlayerController } from "./player.svelte";

	const player = new PlayerController();

	import FavoriteOutline from "~icons/material-symbols/favorite-outline-rounded";
	import Favorite from "~icons/material-symbols/favorite-rounded";
	import PauseIcon from "~icons/material-symbols/pause-rounded";
	import PlayArrowIcon from "~icons/material-symbols/play-arrow-rounded";
	import SkipNextIcon from "~icons/material-symbols/skip-next-rounded";

	onMount(() => player.init());

	async function onPlay() {
		if (player.duration === 0) {
			await player.loadSong("/home/eyewave/Music/cumzo-discum.mp3");
		}

		if (player.isPlaying) player.pause();
		else player.play();
	}

	function parseNumberFromEvent(cb: (value: number) => unknown) {
		return (e: Event & { currentTarget: HTMLInputElement }) => {
			const value = Number(e.currentTarget.value);
			if (Number.isNaN(value)) return;

			cb(value);
		};
	}

	const onChangeVolume = parseNumberFromEvent(v => {
		player.volume = v;
	});

	const onChangeSpeed = parseNumberFromEvent(v => {
		player.playbackSpeed = v;
	});
</script>

<div class="w-full p-4 bg-cya"></div>
