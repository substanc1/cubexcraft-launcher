<script>
  // Голова скина 20x20 без сглаживания (ImageView smooth=false, скругление 8 -> радиус 4): лицо + слой «шляпы».
  // Скин - с сайта (texture/skin/<ник>.png, без скина сайт отдаёт скин по умолчанию); пока грузится - default.png.
  import fallback from '../assets/images/icons/default.png';
  let { username = null, size = 20 } = $props();
  let loaded = $state(null);
  const url = $derived(username ? `https://www.cubexcraft.site/texture/skin/${encodeURIComponent(username)}.png` : null);

  $effect(() => {
    loaded = null;
    if (!url) return;
    const img = new Image();
    img.onload = () => (loaded = { src: url, w: img.naturalWidth, h: img.naturalHeight });
    img.src = url;
  });
  // масштаб: 8 пикселей лица -> size
  const k = $derived(size / 8);
</script>

{#if loaded}
  <span class="av" style="width:{size}px;height:{size}px;
    background-image:url('{loaded.src}'),url('{loaded.src}');
    background-size:{loaded.w * k * (64 / loaded.w)}px {loaded.h * k * (64 / loaded.w)}px;
    background-position:{-40 * k}px {-8 * k}px, {-8 * k}px {-8 * k}px;"></span>
{:else}
  <img class="av" src={fallback} alt="" width={size} height={size} />
{/if}

<style>
  .av { display: block; border-radius: 4px; image-rendering: pixelated; background-repeat: no-repeat; }
</style>
