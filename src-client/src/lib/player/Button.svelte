<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLButtonElement>;

	let { children, class: className, ...props }: Props = $props();
</script>

<button
	class="text-white p-1 text-2xl rounded-2xl bg-cyan-300 hover:bg-white hover:text-cyan-300 cursor-pointer {className}"
	{...props}
>
	{@render children?.()}
</button>
