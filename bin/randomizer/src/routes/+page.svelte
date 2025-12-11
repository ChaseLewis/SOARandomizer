<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { 
    gameStore, 
    settingsStore, 
    uiStore, 
    isGameLoaded,
  } from '$lib/stores/gameStore';

  async function selectAndLoadIso() {
    uiStore.setLoading(true);
    uiStore.clearMessages();

    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'GameCube ISO', extensions: ['iso', 'gcm'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (selected) {
        const result = await gameStore.load(selected as string);
        if (!result.success) {
          uiStore.setError(result.error || 'Failed to load ISO');
        }
      }
    } catch (e) {
      uiStore.setError(`Error: ${e}`);
    } finally {
      uiStore.setLoading(false);
    }
  }

  async function closeIso() {
    await gameStore.close();
    settingsStore.reset();
  }

  function generateSeed() {
    settingsStore.generateRandomSeed();
  }
</script>

<div class="container mx-auto p-8 max-w-4xl">
  {#if !$isGameLoaded}
    <!-- ISO Loader Section -->
    <div class="card preset-filled-surface-50-950 p-8 text-center">
      <div class="flex flex-col items-center gap-6">
        <div class="text-6xl animate-bounce">🌙</div>
        <h2 class="text-3xl font-bold">Welcome, Blue Rogue!</h2>
        <p class="text-surface-600 dark:text-surface-400">
          Load a Skies of Arcadia Legends ISO to begin your adventure
        </p>
        
        <button 
          type="button" 
          class="btn preset-filled-primary-500 text-lg px-8 py-3"
          onclick={selectAndLoadIso}
          disabled={$uiStore.isLoading}
        >
          {#if $uiStore.isLoading}
            <span class="animate-spin">⏳</span>
            <span>Loading...</span>
          {:else}
            <span>📁</span>
            <span>Select ISO File</span>
          {/if}
        </button>

        {#if $uiStore.error}
          <div class="p-4 rounded-lg bg-error-500/20 border border-error-500 text-error-500">
            <p>{$uiStore.error}</p>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Game Loaded Section -->
    <div class="space-y-6">
      <!-- Game Info Card -->
      <div class="card bg-success-500/10 border border-success-500/30 p-6 rounded-xl">
        <header class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <span class="px-3 py-1 rounded-full text-sm bg-success-500 text-white">✓ ISO Loaded</span>
            <span class="font-semibold">{$gameStore?.region}</span>
          </div>
          <button 
            type="button" 
            class="btn preset-outlined-surface-500"
            onclick={closeIso}
          >
            Close ISO
          </button>
        </header>
        <p class="text-sm text-surface-600 dark:text-surface-400 break-all">{$gameStore?.path}</p>
      </div>

      <!-- Randomizer Settings Card -->
      <div class="card preset-filled-surface-50-950 p-6 rounded-xl">
        <header class="mb-6">
          <h3 class="text-2xl font-bold text-primary-500">Randomizer Settings</h3>
          <p class="text-surface-600 dark:text-surface-400 text-sm mt-1">
            Choose what elements to randomize in your adventure
          </p>
        </header>

        <!-- Seed Input -->
        <div class="mb-6">
          <label class="block mb-2 font-medium">Seed</label>
          <div class="flex gap-2">
            <input 
              type="text" 
              class="input flex-1 px-4 py-2 rounded-lg bg-surface-200 dark:bg-surface-800 border border-surface-300 dark:border-surface-700" 
              placeholder="Enter seed or generate..."
              bind:value={$settingsStore.seed}
            />
            <button 
              type="button"
              class="btn preset-filled-secondary-500 px-4 py-2"
              onclick={generateSeed}
            >
              🎲 Random
            </button>
          </div>
          <p class="text-xs text-surface-500 mt-1">
            Use the same seed to get identical randomization results
          </p>
        </div>

        <!-- Toggle Options -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.weapons} 
            />
            <div>
              <p class="font-medium">⚔️ Weapons</p>
              <p class="text-sm text-surface-500">Randomize weapon stats and effects</p>
            </div>
          </label>

          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.armor} 
            />
            <div>
              <p class="font-medium">🛡️ Armor</p>
              <p class="text-sm text-surface-500">Randomize armor stats and properties</p>
            </div>
          </label>

          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.accessories} 
            />
            <div>
              <p class="font-medium">💍 Accessories</p>
              <p class="text-sm text-surface-500">Randomize accessory effects</p>
            </div>
          </label>

          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.shops} 
            />
            <div>
              <p class="font-medium">🏪 Shops</p>
              <p class="text-sm text-surface-500">Randomize shop inventories</p>
            </div>
          </label>

          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.treasureChests} 
            />
            <div>
              <p class="font-medium">📦 Treasure Chests</p>
              <p class="text-sm text-surface-500">Randomize chest contents</p>
            </div>
          </label>

          <label class="flex items-center gap-4 p-4 rounded-lg bg-surface-100 dark:bg-surface-800/50 hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors cursor-pointer">
            <input 
              type="checkbox" 
              class="w-5 h-5 rounded accent-primary-500"
              bind:checked={$settingsStore.enemies} 
            />
            <div>
              <p class="font-medium">👾 Enemies</p>
              <p class="text-sm text-surface-500">Randomize enemy stats and drops</p>
            </div>
          </label>
        </div>

        <!-- Generate Button -->
        <footer class="mt-8 flex justify-center">
          <button 
            type="button" 
            class="btn preset-filled-primary-500 text-lg px-8 py-3"
          >
            <span>✨</span>
            <span>Generate Randomized ISO</span>
          </button>
        </footer>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }
  
  .animate-bounce {
    animation: bounce 2s ease-in-out infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }
</style>
