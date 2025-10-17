export function display(sample: number, sampleRate: number) {
	const n = (sample / sampleRate) | 0;

	const h = (n / 3600) | 0;
	const m = ((n % 3600) / 60) | 0;
	const s = n % 60;

	return `${h > 0 ? `${h}`.padStart(2, '0') + ':' : ''}${`${m}`.padStart(2, '0')}:${`${s}`.padStart(2, '0')}`;
}

export const displayCurrentTime = (duration: number, position: number, sampleRate: number) =>
	display(duration, sampleRate) + ':' + display(position, sampleRate);
