import { registerEnv } from 'mini-van-plate/shared';
import van from 'vanjs-core';

export function onPageLoad(page: () => HTMLElement) {
	if (typeof window === 'undefined') return;

	registerEnv({ van });
	van.hydrate(document.body, page);
}
