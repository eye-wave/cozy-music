import { invoke } from "@tauri-apps/api/core";
import type { LoadSongResult } from "~types/LoadSongResult";
import type { PlayerProps } from "~types/PlayerProps";

export class PlayerController {
	private _isPlaying = $state(false);
	private _playbackSpeed = $state(1.0);
	private _position = $state(0);
	private _volume = $state(1.0);
	private _sampleRate = $state(44_100);
	private _duration = $state(0);
	private _localSampleRate = $state(44_100);
	private _timeoutId: number = -1;

	public async init(): Promise<void> {
		try {
			const props = await invoke<PlayerProps>("player_get_props");

			this._sampleRate = props.sampleRate;
			this._isPlaying = props.isPlaying;
			this._playbackSpeed = props.playbackSpeed;
			this._position = props.position;
			this._volume = props.volume;
			this._duration = props.localDuration;
			this._localSampleRate = props.localSampleRate;

			if (props.isPlaying) {
				this.startPositionTimer();
			}
		} catch (_) {}
	}

	public get playbackSpeed(): number {
		return this._playbackSpeed;
	}

	public set playbackSpeed(speed: number) {
		this.invokeCommand("player_set_playback_speed", { speed }).then(() => {
			this._playbackSpeed = speed;
		});
	}

	public get volume(): number {
		return this._volume;
	}

	public set volume(volume: number) {
		this.invokeCommand("player_set_volume", { volume }).then(() => {
			this._volume = volume;
		});
	}

	public get position(): number {
		return this._position;
	}

	public set position(seconds: number) {
		this.invokeCommand("player_set_position", { secs: seconds }).then(() => {
			this._position = seconds * this.playbackRate * this.sampleRate;
		});
	}

	public async loadSong(path: string): Promise<void> {
		try {
			const props = await invoke<LoadSongResult>("player_load_song", { path });
			this._duration = props.duration;
			this._localSampleRate = props.sampleRate;
		} catch (_) {}
	}

	public async play(): Promise<void> {
		try {
			await invoke("player_play");
			this._isPlaying = true;
			this.startPositionTimer();
		} catch (_) {}
	}

	public async pause(): Promise<void> {
		try {
			await invoke("player_pause");
			this.stopPlayback();
		} catch (_) {}
	}

	public async stop(): Promise<void> {
		try {
			await invoke("player_stop");
			this.stopPlayback();
		} catch (_) {}
	}

	private async invokeCommand(
		command: string,
		args?: Record<string, unknown>,
	): Promise<void> {
		try {
			await invoke(command, args);
		} catch (_) {}
	}

	private stopPlayback(): void {
		this._isPlaying = false;
		this.clearPositionTimer();
	}

	private startPositionTimer(): void {
		this.clearPositionTimer();
		this._timeoutId = window.setTimeout(
			() => this.updatePosition(),
			this.timerInterval,
		);
	}

	private clearPositionTimer(): void {
		if (this._timeoutId === -1) return;

		clearTimeout(this._timeoutId);
		this._timeoutId = -1;
	}

	private async updatePosition(): Promise<void> {
		try {
			const position = await invoke<number>("player_get_position");
			this._position = position;
		} catch (_) {}

		this.startPositionTimer();
	}

	private get timerInterval(): number {
		return this.playbackRate * 100;
	}

	private formatTime(seconds: number): string {
		const minutes = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
	}

	private toSeconds(value: number) {
		return value / (this.sampleRate * this.playbackRate);
	}

	public duration = $derived(this._duration);
	public isPlaying = $derived(this._isPlaying);
	public sampleRate = $derived(this._sampleRate);
	public playbackRate = $derived(
		(this._localSampleRate / this._sampleRate) * this._playbackSpeed,
	);
	public timeCode = $derived(
		`${this.formatTime(this.toSeconds(this._position))} / ${this.formatTime(this.toSeconds(this._duration))}`,
	);
}
