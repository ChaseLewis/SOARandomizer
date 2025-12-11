<script lang="ts">
  import '../app.css';

  let { children } = $props();
</script>

<div class="min-h-screen bg-surface-950" data-theme="cerberus">
  <!-- Header -->
  <header class="bg-surface-900 border-b border-surface-800">
    <div class="container mx-auto px-4">
      <div class="flex items-center justify-between h-16">
        <div class="flex items-center gap-3">
          <span class="text-2xl">⚓</span>
          <h1 class="text-xl font-bold text-primary-500">
            Skies of Arcadia Randomizer
          </h1>
        </div>
        <span class="text-sm text-surface-400">v0.1.0</span>
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="py-8">
    {@render children()}
  </main>

  <!-- Footer -->
  <footer class="border-t border-surface-800 py-4 text-center text-surface-500 text-sm">
    Powered by ALX · Made with 💙
  </footer>
</div>
