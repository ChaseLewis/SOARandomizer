import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types
export interface GameInfo {
  version: string;
  region: string;
  path: string;
}

export interface RandomizerSettings {
  weapons: boolean;
  armor: boolean;
  accessories: boolean;
  shops: boolean;
  treasureChests: boolean;
  enemies: boolean;
  seed: string;
}

export interface CommandResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

// Default settings
const defaultSettings: RandomizerSettings = {
  weapons: false,
  armor: false,
  accessories: false,
  shops: false,
  treasureChests: false,
  enemies: false,
  seed: '',
};

// Create stores
function createGameStore() {
  const { subscribe, set, update } = writable<GameInfo | null>(null);

  return {
    subscribe,
    set,
    clear: () => set(null),
    load: async (path: string): Promise<CommandResult<GameInfo>> => {
      const result: CommandResult<GameInfo> = await invoke('load_iso', { path });
      if (result.success && result.data) {
        set(result.data);
      }
      return result;
    },
    close: async () => {
      await invoke('close_iso');
      set(null);
    },
  };
}

function createSettingsStore() {
  const { subscribe, set, update } = writable<RandomizerSettings>(defaultSettings);

  return {
    subscribe,
    set,
    reset: () => set(defaultSettings),
    toggle: (key: keyof RandomizerSettings) => {
      update(settings => ({
        ...settings,
        [key]: !settings[key],
      }));
    },
    setSeed: (seed: string) => {
      update(settings => ({
        ...settings,
        seed,
      }));
    },
    generateRandomSeed: () => {
      const seed = Math.random().toString(36).substring(2, 10).toUpperCase();
      update(settings => ({
        ...settings,
        seed,
      }));
    },
  };
}

function createUIStore() {
  const { subscribe, set, update } = writable({
    isLoading: false,
    error: '',
    successMessage: '',
  });

  return {
    subscribe,
    setLoading: (isLoading: boolean) => update(s => ({ ...s, isLoading })),
    setError: (error: string) => update(s => ({ ...s, error, successMessage: '' })),
    setSuccess: (successMessage: string) => update(s => ({ ...s, successMessage, error: '' })),
    clearMessages: () => update(s => ({ ...s, error: '', successMessage: '' })),
  };
}

// Export stores
export const gameStore = createGameStore();
export const settingsStore = createSettingsStore();
export const uiStore = createUIStore();

// Derived stores
export const isGameLoaded = derived(gameStore, $game => $game !== null);

