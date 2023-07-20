<script>
	import Files from '../lib/Files.svelte';

	let outputFolder = 'root';
	let saveInSameFolder = false;
</script>

<label for="checkbox-1">
	<input type="checkbox" id="checkbox-1" name="checkbox-1" bind:checked={saveInSameFolder} />
	Save to original folder
</label>
{#if !saveInSameFolder}
	<label for="outputLocation"
		>Output Location
		<button style="width: auto; height: auto;" class="secondary">Choose Folder...</button>
	</label>
{/if}
<button style="width: 150px;">Export</button>

<hr />

<Files />
