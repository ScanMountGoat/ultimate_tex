<script>
	import { invoke } from '@tauri-apps/api/tauri';
	import { emit, listen } from '@tauri-apps/api/event'
	import { onMount } from 'svelte';

	let fileTypes = ['Dds', 'Png', 'Tiff', 'Nutexb', 'Bntx'];
	let formatTypes = [
		'R8Unorm',
		'R8G8B8A8Unorm',
		'R8G8B8A8Srgb',
		'R32G32B32A32Float',
		'B8G8R8A8Unorm',
		'B8G8R8A8Srgb',
		'BC1Unorm',
		'BC1Srgb',
		'BC2Unorm',
		'BC2Srgb',
		'BC3Unorm',
		'BC3Srgb',
		'BC4Unorm',
		'BC4Snorm',
		'BC5Unorm',
		'BC5Snorm',
		'BC6Ufloat',
		'BC6Sfloat',
		'BC7Unorm',
		'BC7Srgb'
	];
	let mipmapTypes = ['Disabled', 'FromSurface', 'GeneratedAutomatic'];
	let compressionTypes = ['Fast', 'Normal', 'Slow'];

	// Reduced options for global presets.
	let presetFileTypes = ['PNG', 'DDS', 'Nutexb', 'Bntx', 'Custom...'];
	let presetFormatTypes = ['Color (sRGB) + Alpha', 'Color (Linear) + Alpha', 'Custom...'];
	let presetMipmapTypes = ['Enabled', 'Disabled', 'Custom...'];
	let presetCompressionTypes = ['Fast', 'Normal', 'Slow', 'Custom...'];

	let items = [];

	async function loadList() {
		items = await invoke('load_items', {});

		// TODO: Where to call this?
		await listen('items_changed', async (event) => {
			items = await invoke('load_items', {});
		});
	}

	onMount(loadList);
</script>

<div class="flex-container">
	<fieldset>
		<legend><strong>Output Type</strong></legend>
		{#each presetFileTypes as option}
			<label for="outputType">
				<input type="radio" id="radio-1" name="outputType" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Output Format</strong></legend>
		{#each presetFormatTypes as option}
			<label for="outputFormat">
				<input type="radio" id="radio-1" name="outputFormat" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Mipmaps</strong></legend>
		{#each presetMipmapTypes as option}
			<label for="mipmaps">
				<input type="radio" id="radio-1" name="mipmaps" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Compression</strong></legend>
		{#each presetCompressionTypes as option}
			<label for="compression">
				<input type="radio" id="radio-1" name="compression" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
</div>

<figure>
	<table role="grid">
		<thead>
			<tr>
				<th scope="col"><strong>Name</strong></th>
				<!-- <th scope="col"><strong>Type</strong></th> -->
				<th scope="col"><strong>Format</strong></th>
				<th scope="col"><strong>Size</strong></th>
				<th scope="col"><strong>Output Type</strong></th>
				<th scope="col"><strong>Output Format</strong></th>
				<th scope="col"><strong>Compression</strong></th>
				<th scope="col"><strong>Mipmaps</strong></th>
				<th />
			</tr>
		</thead>
		<tbody>
			{#each items as item}
				<tr>
					<th scope="row">{item.name}</th>
					<!-- <th>{item.file_type}</th> -->
					<th>{item.format}</th>
					<th>{item.dimensions}</th>
					<th>
						<select bind:value={item.file_type}>
							{#each fileTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select bind:value={item.format}>
							{#each formatTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select bind:value={item.quality}>
							{#each compressionTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select bind:value={item.mipmaps}>
							{#each mipmapTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<button class="secondary">Remove</button>
					</th>
				</tr>
			{/each}
		</tbody>
	</table>
</figure>

<style>
	.flex-container {
		display: grid;
		grid-template-columns: 150px 150px 150px 150px;
	}
</style>
