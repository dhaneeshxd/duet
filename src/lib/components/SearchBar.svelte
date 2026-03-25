<script lang="ts">
    import { searchQuery, searchTracks, loadTracks } from "$lib/stores/library";
    import { debounce } from "$lib/utils";

    let value = "";

    const handleSearch = debounce(async (q: string) => {
        searchQuery.set(q);
        if (q.trim()) {
            await searchTracks(q);
        } else {
            await loadTracks();
        }
    }, 300);

    function onInput(e: Event) {
        value = (e.target as HTMLInputElement).value;
        handleSearch(value);
    }

    function clear() {
        value = "";
        handleSearch("");
    }
</script>

<div class="search-wrap">
    <span class="icon">⌕</span>
    <input
        type="text"
        placeholder=" search..."
        {value}
        on:input={onInput}
    />
    {#if value}
        <button class="clear" on:click={clear}>✕</button>
    {/if}
</div>

<style>
    .search-wrap {
        position: relative;
        display: flex;
        align-items: center;
        background: #1a1a1a;
        border: 1px solid #2a2a2a;
        border-radius: 12px;
        height: 28px;
        padding: 0 8px;
        gap: 6px;

        width: 180px;
        transition: border-color 0.2s;
    }

    .search-wrap:focus-within {
        border-color: #f59e0b;
    }

    .icon {
        color: #666;
        font-size: 22px;
        pointer-events: none;
        margin-bottom: 4px;
    }

    input {
        flex: 1;
        background: none;
        border: none;
        outline: none;
        color: #e8e8e8;
        font-size: 14px;
        padding: 10px 0;
        margin-top: 1px;
    }

    input::placeholder {
        color: #555;
    }

    .clear {
        background: none;
        border: none;
        color: #555;
        cursor: pointer;
        font-size: 12px;
        padding: 4px;
        transition: color 0.15s;
    }

    .clear:hover {
        color: #fff;
    }
</style>
