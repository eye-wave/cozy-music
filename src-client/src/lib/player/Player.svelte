<script lang="ts">
	import { onMount } from "svelte";
	import Button from "./Button.svelte";
	import { PlayerController } from "./player.svelte";

	const player = new PlayerController();

	import FavoriteOutlineIcon from "~icons/material-symbols/favorite-outline-rounded";
	import FavoriteIcon from "~icons/material-symbols/favorite-rounded";
	import PauseIcon from "~icons/material-symbols/pause-rounded";
	import PlayIcon from "~icons/material-symbols/play-arrow-rounded";
	import SkipIcon from "~icons/material-symbols/skip-next-rounded";

	let isCurrentLoved = $state(false);

	let isDragging = $state(false);
	let visiblePosition = $state(player.position);

	$effect(() => {
		if (isDragging) return;

		visiblePosition = player.position;
	});

	onMount(() => player.init());

	async function onPlay() {
		if (player.duration === 0) {
			await player.loadSong("/home/eyewave/Music/cumzo-discum.mp3");
		}

		if (player.isPlaying) player.pause();
		else player.play();
	}

	function parseNumberFromEvent(
		cb: (value: number) => unknown,
		fallback?: () => void,
	) {
		return (e: Event & { currentTarget: HTMLInputElement }) => {
			fallback?.();

			const value = Number(e.currentTarget.value);
			if (Number.isNaN(value)) return;

			cb(value);
		};
	}

	const onSeek = parseNumberFromEvent(
		pos => {
			const seconds = pos / player.playbackRate / player.sampleRate;
			player.position = seconds;
		},
		() => {
			isDragging = false;
		},
	);

	const onChangeVolume = parseNumberFromEvent(v => {
		player.volume = v;
	});

	const onChangeSpeed = parseNumberFromEvent(v => {
		player.playbackSpeed = v;
	});
</script>

<div class="w-full p-4 bg-cyan-500 ">
	<div class="flex items-center gap-2 justify-center">
		<Button>
			{#if isCurrentLoved}
			<FavoriteIcon/>
			{:else}
			<FavoriteOutlineIcon/>
			{/if}
		</Button>
		<Button>
			<SkipIcon style="transform:scaleX(-1)"/>
		</Button>
		<Button onclick={onPlay}>
			{#if player.isPlaying}
			<PauseIcon/>
			{:else}
			<PlayIcon/>
			{/if}
		</Button>
		<Button>
			<SkipIcon/>
		</Button>

		<p class="select-none text-white font-semibold font-mono">
			{player.timeCode}
		</p>
	</div>

	<div>
		<input
			onmousedown={() => isDragging = true}
			onmouseleave={() => isDragging = false}
			onmouseup={onSeek}
			class="w-full"
			type="range"
			value={visiblePosition}
			max={player.duration}
		>
	</div>
</div>
