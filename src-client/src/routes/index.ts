import { onPageLoad } from '../components/hydrate';
import { env } from 'mini-van-plate/shared';

export default function Page() {
	const { ul, li } = env.van.tags;

	const items = env.van.state(Array.from({ length: 10 }, (_, i) => i));
	const dummy = env.van.state(0);

	return ul(
		{ class: 'p-4 bg-stone-800 flex flex-wrap items-center gap-2' },
		items.val.map((n, i) =>
			li(
				{
					class: 'bg-stone-600 p-4 rounded-lg cursor-pointer',
					onclick: () => {
						items.val[i] += 2;
						dummy.val += 1;
					}
				},
				items.val[i]
			)
		)
	);
}

onPageLoad(Page);
