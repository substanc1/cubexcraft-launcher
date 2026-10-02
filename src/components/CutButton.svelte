<script>
  // Кнопка со срезанными углами (#authButton / #playButton: -fx-shape + -fx-scale-shape).
  // Форма из старого CSS задана в своих единицах и растягивается на размер кнопки - здесь то же самое
  // через SVG с preserveAspectRatio="none"; рамка 1px не растягивается (non-scaling-stroke).
  let {
    shape = 'M0,0 L358,0 L372,14 L372,46 L14,46 L0,32 Z',
    viewBox = '0 0 372 46',
    variant = 'active', // active | unactive | error | disabled
    onclick = null,
    disabled = false,
    class: cls = '',
    style = '',
    children,
  } = $props();
</script>

<button type="button" class="cut {variant} {cls}" {style} {disabled} onclick={onclick} data-no-drag>
  <svg class="shape" {viewBox} preserveAspectRatio="none" aria-hidden="true">
    <path d={shape} vector-effect="non-scaling-stroke" />
  </svg>
  <span class="label">{@render children?.()}</span>
</button>

<style>
  .cut {
    position: relative; display: grid; place-items: center;
    width: 100%; cursor: pointer; isolation: isolate;
  }
  .shape { position: absolute; inset: 0; width: 100%; height: 100%; z-index: -1; overflow: visible; }
  .shape path { stroke-width: 1px; }
  .label { position: relative; }

  /* AuthButtonState: activeButton / unactiveButton / errorButton */
  .active .shape path, .unactive .shape path { fill: var(--redTr); stroke: var(--red); }
  .active, .unactive { color: var(--text-hover); filter: drop-shadow(0 0 8px var(--glow-color)); }
  .active:hover .shape path, .unactive:hover .shape path { fill: var(--redTr-hover); }
  .unactive { opacity: 0.75; }
  .unactive:hover { opacity: 1; }
  .error .shape path { fill: var(--yellowTr); stroke: var(--yellow); }
  .error { color: var(--yellow); }
  /* .playButton / .playButtonDisabled (servermenu.css) */
  .play .shape path { fill: var(--redTr); stroke: var(--red); }
  .play { color: #FFD6D9; filter: drop-shadow(0 0 8px var(--glow-color)); }
  .play:hover .shape path { fill: var(--redTr-hover); }
  .disabled .shape path { fill: var(--panel-flat); stroke: var(--border-dark); }
  .disabled { color: var(--text-secondary); cursor: default; }
</style>
