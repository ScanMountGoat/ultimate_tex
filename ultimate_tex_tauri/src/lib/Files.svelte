<script>
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	let outputTypes = ['PNG', 'DDS', 'Nutexb', 'Custom...'];
	let outputFormats = ['A', 'B', 'C'];
	let mipmaps = ['A', 'B', 'C'];
	let compressions = ['Slow', 'Normal', 'Fast'];

	let selected = 'DDS';

	let items = [];

	async function loadList() {
		items = await invoke('load_items', {});
	}

	onMount(loadList);
</script>

<div class="flex-container">
	<fieldset>
		<legend><strong>Output Type</strong></legend>
		{#each outputTypes as option}
			<label for="radio-1">
				<input type="radio" id="radio-1" name="radio" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Output Format</strong></legend>
		{#each outputFormats as option}
			<label for="radio-1">
				<input type="radio" id="radio-1" name="radio" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Mipmaps</strong></legend>
		{#each mipmaps as option}
			<label for="radio-1">
				<input type="radio" id="radio-1" name="radio" value={option} />
				{option}
			</label>
		{/each}
	</fieldset>
	<fieldset>
		<legend><strong>Compression</strong></legend>
		{#each compressions as option}
			<label for="radio-1">
				<input type="radio" id="radio-1" name="radio" value={option} />
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
						<select>
							<option value="nutexb">{item.file_type}</option>
						</select>
					</th>
					<th>
						<select>
							<option value="nutexb">{item.format}</option>
						</select>
					</th>
					<th>
						<select>
							<option value="fast">{item.quality}</option>
						</select>
					</th>
					<th>
						<select name="mipmaps" id="mipmaps">
							<option value="fast">{item.mipmaps}</option>
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
