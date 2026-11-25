<script>
  import { parseRich } from './rich.js';
  let { text = '' } = $props();
</script>{#each parseRich(text) as tok}{#if tok.kind === 'b'}<span class="acc-b">{tok.text}</span>{:else if tok.kind === 'neon'}<span class="acc-neon">{tok.text}</span>{:else}{tok.text}{/if}{/each}
