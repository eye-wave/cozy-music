<script lang="ts">
	import { displayCurrentTime } from './display';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import type { PlayerProps } from '~types/PlayerProps';

	let isPlaying = $state(false);
	let duration = $state(0);
	let position = $state(0);
	let sampleRate = $state(44100);

	onMount(() => {
		initValues();
		setInterval(updatePosition, 500);
	});

	function initValues() {
		invoke('get_samplerate').then((sr) => {
			sampleRate = sr as number;
		});

		invoke('player_get_props').then((p) => {
			const props = p as PlayerProps;

			volume = props.volume;
			speed = props.playbackSpeed;
		});
	}

	const updatePosition = () =>
		isPlaying &&
		invoke('get_position').then((p) => {
			position = p as number;
		});

	let isSongLoaded = $state(false);

	let volume = $state(0);
	let speed = $state(0);

	function onChangeVolume(e: Event & { currentTarget: HTMLInputElement }) {
		volume = Number(e.currentTarget.value);

		invoke('player_set_volume', { volume });
	}

	function onChangeSpeed(e: Event & { currentTarget: HTMLInputElement }) {
		speed = Number(e.currentTarget.value);

		invoke('player_set_playback_speed', { speed });
	}

	async function onPlay() {
		try {
			if (!isPlaying) {
				if (!isSongLoaded) {
					const path = '/home/eyewave/Music/cumzo-discum.mp3';

					duration = await invoke('load_song', { path });
					isSongLoaded = true;
				}

				await invoke('player_play');

				isPlaying = true;
				return;
			}

			await invoke('player_pause');
			isPlaying = false;
		} catch (err) {
			console.error(err);
		}
	}
</script>

<div class="bg-stone-700 m-2 h-52">
	<button class="bg-white text-black px-4 rounded-full" onclick={onPlay}
		>{isPlaying ? 'Stop' : 'Play'}</button
	>

	<label
		><p>{volume}</p>
		<input type="range" min="0.0" max="1.0" oninput={onChangeVolume} step="0.01" />
	</label>
	<label
		><p>{speed}</p>
		<input type="range" min="0.1" max="1.5" value="1.0" oninput={onChangeSpeed} step="0.01" />
	</label>

	<p>{displayCurrentTime(duration, position, sampleRate)}</p>
</div>
