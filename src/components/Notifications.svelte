<script>
  // FxStage.pushNotification: колонка справа (10px от краёв), снизу вверх, между карточками 10px;
  // карточка .notification 360px, иконка-колокольчик 48x48, заголовок/текст (components.css)
  import { app } from '../lib/app.svelte.js';
</script>

<div class="notifications">
  {#each app.notifications as n (n.id)}
    <div class="notification">
      <div class="icon">
        <svg viewBox="4.4 2 24.1 28" width="24" height="28" aria-hidden="true">
          <path d="M16.466 30a4 4 0 0 0 3.992-4.251.46.46 0 0 0-.465-.42l-7.037-.08a.46.46 0 0 0-.476.41A4 4 0 0 0 16.466 30ZM7.133 12c0-5.155 4.178-10 9.333-10s9.333 4.845 9.333 10v3.28c0 1.73.74 3.377 2.033 4.527 1.387 1.233.515 3.526-1.341 3.526H6.44c-1.856 0-2.728-2.293-1.34-3.526a6.055 6.055 0 0 0 2.032-4.526V12Z" />
        </svg>
      </div>
      <div class="body">
        <div class="heading">{n.head}</div>
        <div class="text">{n.message}</div>
      </div>
    </div>
  {/each}
</div>

<style>
  .notifications {
    position: absolute; right: 10px; top: 10px; bottom: 10px; z-index: 50;
    display: flex; flex-direction: column; justify-content: flex-end; align-items: flex-end; gap: 10px;
    pointer-events: none;
  }
  .notification {
    width: 360px; display: flex; pointer-events: auto;
    padding: 12px 14px;
    background: var(--panel-flat);
    border: 1px solid var(--yellow);
    box-shadow: var(--dropshadow);
  }
  .icon { width: 48px; height: 48px; flex: none; display: grid; place-items: center; }
  .icon path { fill: var(--header); }
  .body { padding: 0 10px; display: flex; flex-direction: column; gap: 5px; justify-content: center; min-width: 0; }
  .heading { max-width: 256px; color: var(--yellow); font-family: 'JetBrains Mono Bold'; font-size: 12px; }
  .text { max-width: 256px; color: var(--text-secondary); font-family: 'JetBrains Mono'; font-size: 11px; overflow-wrap: anywhere; }
</style>
