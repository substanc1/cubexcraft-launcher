<script>
  // .icon-button шапки: 30x30, рамка -fx-border-dark, глиф JetBrains Mono 13px
  import { tooltip } from '../lib/tooltip.js';
  let { glyph, kind = '', tip = '', onclick = null } = $props();
</script>

<button type="button" class="icon-button {kind}" use:tooltip={tip} onclick={onclick} data-no-drag>{glyph}</button>
