<script lang="ts">
	import Counter from '$lib/components/Counter.svelte';
</script>

<svelte:head>
	<title>RankChoice.app</title>
	<meta name="description" content="Ranked-Choice Voting Platform" />
</svelte:head>

<div class="min-h-screen bg-gray-50 py-12">
	<div class="max-w-4xl mx-auto px-4">
		<header class="text-center mb-12">
			<h1 class="text-4xl font-bold text-gray-900 mb-4">
				🗳️ RankChoice.app
			</h1>
			<p class="text-xl text-gray-600 mb-2">
				Modern Ranked-Choice Voting Platform
			</p>
			<p class="text-lg text-gray-500">
				Built with Svelte 5 & Rust
			</p>
		</header>

		<div class="grid gap-8 md:grid-cols-2">
			<div class="bg-white rounded-lg shadow-lg p-6">
				<h2 class="text-2xl font-semibold text-gray-900 mb-4">✨ Features</h2>
				<ul class="space-y-2 text-gray-600">
					<li>• Create ranked-choice voting polls</li>
					<li>• Drag-and-drop ballot interface</li>
					<li>• Real-time results visualization</li>
					<li>• Email distribution system</li>
					<li>• Secure & anonymous voting</li>
					<li>• Analytics dashboard</li>
				</ul>
			</div>

			<div class="bg-white rounded-lg shadow-lg p-6">
				<h2 class="text-2xl font-semibold text-gray-900 mb-4">🚀 Tech Stack</h2>
				<ul class="space-y-2 text-gray-600">
					<li>• <strong>Frontend:</strong> Svelte 5 + TypeScript</li>
					<li>• <strong>Backend:</strong> Rust + Axum</li>
					<li>• <strong>Database:</strong> PostgreSQL</li>
					<li>• <strong>Deployment:</strong> AWS Serverless</li>
					<li>• <strong>Styling:</strong> Tailwind CSS</li>
					<li>• <strong>Infrastructure:</strong> Terraform</li>
				</ul>
			</div>
		</div>

		<div class="mt-12">
			<h2 class="text-2xl font-semibold text-gray-900 text-center mb-8">
				Svelte 5 Demo (with Runes)
			</h2>
			<Counter />
		</div>

		<div class="mt-12 text-center">
			<div class="space-y-4">
				<h3 class="text-xl font-semibold text-gray-900">Status</h3>
				<div class="flex justify-center gap-4">
					<div class="bg-green-100 text-green-800 px-3 py-1 rounded-full text-sm font-medium">
						✅ Backend Running (Rust)
					</div>
					<div class="bg-green-100 text-green-800 px-3 py-1 rounded-full text-sm font-medium">
						✅ Frontend Running (Svelte 5)
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
