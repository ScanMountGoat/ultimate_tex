<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { emit, listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	// Initialized from Rust enum variants.
	let fileTypes = [];
	let formatTypes = [];
	let mipmapTypes = [];
	let compressionTypes = [];

	// Reduced options for global presets.
	let presetFileTypes = ['Png', 'Dds', 'Nutexb', 'Bntx'];
	let presetFormatTypes = [['BC7Srgb', 'Color (sRGB) + Alpha'], ['BC7Unorm', 'Color (Linear) + Alpha']];
	let presetMipmapTypes = [['GeneratedAutomatic', 'Enabled'], ['Disabled', 'Disabled']];
	let presetCompressionTypes = ['Fast', 'Normal', 'Slow'];

	// TODO: Better way to just have Rust initialize this?
	let saveInSameFolder = false;
	let outputFolder = null;

	// TODO: set proper defaults.
	let overrides = {
		outputFileType: null,
		outputFormat: null,
		mipmaps: null,
		compressionQuality: null
	};

	let fileSettings = [];

	let isExporting = false;

	let footerMessages = [];

	async function initializeApp() {
		fileTypes = await invoke('image_file_type_variants', {});
		formatTypes = await invoke('image_format_variants', {});
		mipmapTypes = await invoke('mipmaps_variants', {});
		compressionTypes = await invoke('quality_variants', {});

		fileSettings = await invoke('load_files', {});

		await listen('files_changed', async (event) => {
			fileSettings = await invoke('load_files', {});
		});
		await listen('output_folder_changed', async (event) => {
			outputFolder = event.payload;
		});
	}

	onMount(initializeApp);

	async function exportFiles(_) {
		// Pass the AppSettings to Rust in case anything changed.
		let settings = { outputFolder, saveInSameFolder, overrides, fileSettings };
		console.log(fileSettings);
		// Disable the export button until the export completes.
		isExporting = true;
		footerMessages = await invoke('export_files', { settings });
		isExporting = false;
	}

	async function addFiles(_) {
		await invoke('add_files', {});
	}

	async function clearFiles(_) {
		await invoke('clear_files', {});
	}

	async function optimizeNutexb(_) {
		await invoke('optimize_nutexb', {});
	}

	async function openWiki(_) {
		await invoke('open_wiki', {});
	}

	async function selectFolder(_) {
		outputFolder = await invoke('select_output_folder', {});
	}

	function formatDimensions(dimensions: [number, number, number]): string {
		let [w, h, d] = dimensions;
		return `${w}x${h}x${d}`;
	}

	function isCompressed(item: any): boolean {
		let fileType = overrides.outputFileType ?? item.outputFileType;
		return isCompressedType(fileType);
	}

	function isCompressedType(fileType: string): boolean {
		return fileType != 'Png' && fileType != 'Tiff';
	}

	window.onclick = function (e) {
		// Close menus when clicking menu options.
		if (e.target.tagName == 'A') {
			for (const element of document.getElementsByTagName('details')) {
				element.open = false;
			}
		}
	};
</script>

<nav>
	<ul>
		<li>
			<details role="list" dir="ltr">
				<summary aria-haspopup="listbox" role="link">File</summary>
				<ul role="listbox">
					<li><a href="#top" on:click={addFiles}>Add Files...</a></li>
					<li><a href="#top" on:click={clearFiles}>Clear Files</a></li>
				</ul>
			</details>
		</li>
		<li>
			<details role="list" dir="ltr">
				<summary aria-haspopup="listbox" role="link">Batch</summary>
				<ul role="listbox">
					<li><a href="#top" on:click={optimizeNutexb}>Optimize Nutexb Padding...</a></li>
				</ul>
			</details>
		</li>
		<li>
			<details role="list" dir="ltr">
				<summary aria-haspopup="listbox" role="link">Help</summary>
				<ul role="listbox">
					<li><a href="#top" on:click={openWiki}>Wiki</a></li>
				</ul>
			</details>
		</li>
	</ul>
</nav>
<hr />

<label for="saveInSameFolder">
	<input
		type="checkbox"
		id="saveInSameFolder"
		name="saveInSameFolder"
		bind:checked={saveInSameFolder}
	/>
	Save to original folder
</label>
{#if !saveInSameFolder}
	<div class="file-container">
		<button style="width: auto;" class="secondary" on:click={selectFolder}>
			Select Folder...
		</button>
		<div class="file-text">
			{outputFolder ?? 'No folder selected'}
		</div>
	</div>
{/if}
<button
	style="width: 150px;"
	on:click={exportFiles}
	disabled={(outputFolder == null && !saveInSameFolder) || isExporting}
	>Export
</button>

<hr />

<div class="flex-container">
	<fieldset>
		<legend><strong>Output Type</strong></legend>
		{#each presetFileTypes as option}
			<label for="outputType{option}">
				<input
					type="radio"
					bind:group={overrides.outputFileType}
					id="outputType{option}"
					name="outputType"
					value={option}
				/>
				{option}
			</label>
		{/each}
		<label for="outputTypeNull">
			<input
				type="radio"
				bind:group={overrides.outputFileType}
				id="outputTypeNull"
				name="outputType"
				value={null}
			/>
			Custom...
		</label>
	</fieldset>
	<fieldset disabled={!isCompressedType(overrides.outputFileType)}>
		<legend><strong>Output Format</strong></legend>
		{#each presetFormatTypes as [option, label]}
			<label for="outputFormat{option}">
				<input
					type="radio"
					bind:group={overrides.outputFormat}
					id="outputFormat{option}"
					name="outputFormat"
					value={option}
				/>
				{label}
			</label>
		{/each}
		<label for="outputFormatNull">
			<input
				type="radio"
				bind:group={overrides.outputFormat}
				id="outputFormatNull"
				name="outputFormat"
				value={null}
			/>
			Custom...
		</label>
	</fieldset>
	<fieldset disabled={!isCompressedType(overrides.outputFileType)}>
		<legend><strong>Mipmaps</strong></legend>
		{#each presetMipmapTypes as [option, label]}
			<label for="mipmaps{option}">
				<input
					type="radio"
					bind:group={overrides.mipmaps}
					id="mipmaps{option}"
					name="mipmaps"
					value={option}
				/>
				{label}
			</label>
		{/each}
		<label for="mipmapsNull">
			<input
				type="radio"
				bind:group={overrides.mipmaps}
				id="mipmapsNull"
				name="mipmaps"
				value={null}
			/>
			Custom...
		</label>
	</fieldset>
	<fieldset disabled={!isCompressedType(overrides.outputFileType)}>
		<legend><strong>Compression</strong></legend>
		{#each presetCompressionTypes as option}
			<label for="compression{option}">
				<input
					type="radio"
					bind:group={overrides.compressionQuality}
					id="compression{option}"
					name="compression"
					value={option}
				/>
				{option}
			</label>
		{/each}
		<label for="compressionNull">
			<input
				type="radio"
				bind:group={overrides.compressionQuality}
				id="compressionNull"
				name="compression"
				value={null}
			/>
			Custom...
		</label>
	</fieldset>
</div>

<figure>
	<table role="grid">
		<thead>
			<tr>
				<th scope="col"><strong>Name</strong></th>
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
			{#each fileSettings as item, index}
				<tr>
					<th scope="row">{item.name}</th>
					<th>{item.format}</th>
					<th>{formatDimensions(item.dimensions)}</th>
					<th>
						<select bind:value={item.outputFileType} disabled={overrides.outputFileType != null}>
							{#each fileTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select
							bind:value={item.outputFormat}
							disabled={overrides.outputFormat != null || !isCompressed(item)}
						>
							{#each formatTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select
							bind:value={item.outputQuality}
							disabled={overrides.compressionQuality != null || !isCompressed(item)}
						>
							{#each compressionTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<select
							bind:value={item.outputMipmaps}
							disabled={overrides.mipmaps != null || !isCompressed(item)}
						>
							{#each mipmapTypes as option}
								<option value={option}>{option}</option>
							{/each}
						</select>
					</th>
					<th>
						<button
							class="secondary"
							on:click={(_) => {
								emit('remove_item', index);
							}}>Remove</button
						>
					</th>
				</tr>
			{/each}
		</tbody>
	</table>
	{#if fileSettings.length == 0}
		<div style="text-align: center">
			Drag and drop image files onto the window or add files using File > Add Files...
		</div>
	{/if}
</figure>

<footer>
	<hr />
	{#each footerMessages as message}
		{message}
		<br>
	{/each}
</footer>

<style>
	footer {
		position: fixed;
		bottom: 0;
		width: 100%;
		margin-bottom: 10px;
	}

	nav {
		height: 15px;
	}

	.file-container {
		display: flex;
		justify-content: start;
		align-content: center;
		align-items: center;
	}

	.file-text {
		color: var(--muted-color);
		height: 100%;
		text-align: center;
		margin: 5px;
	}

	.flex-container {
		display: grid;
		grid-template-columns: 125px 175px 125px 150px;
	}

	a,
	[role='link'] {
		--color: var(--color);
	}
	a:is([aria-current], :hover, :active, :focus),
	[role='link']:is([aria-current], :hover, :active, :focus) {
		--color: var(--color);
		--background-color: var(--secondary-focus);
	}
	a:focus,
	[role='link']:focus {
		--color: var(--color);
		--background-color: var(--secondary-focus);
	}
	details summary:focus:not([role='button']) {
		color: var(--color);
	}
</style>
