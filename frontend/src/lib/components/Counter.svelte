<script lang="ts">
	// Demo Svelte 5 component using runes
	let count = $state(0);
	let doubled = $derived(count * 2);
	
	function increment() {
		count += 1;
	}
	
	function decrement() {
		count -= 1;
	}
</script>

<div class="p-6 max-w-sm mx-auto bg-white rounded-xl shadow-lg">
	<h2 class="text-xl font-bold text-gray-900 mb-4">Svelte 5 Counter Demo</h2>
	
	<div class="space-y-4">
		<div class="text-center">
			<p class="text-lg">Count: <span class="font-mono font-bold text-blue-600">{count}</span></p>
			<p class="text-sm text-gray-600">Doubled: <span class="font-mono">{doubled}</span></p>
		</div>
		
		<div class="flex gap-2 justify-center">
			<button 
				onclick={decrement}
				class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition"
			>
				-
			</button>
			<button 
				onclick={increment}
				class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 transition"
			>
				+
			</button>
		</div>
	</div>
</div> 