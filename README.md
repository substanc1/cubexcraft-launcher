# CubeXCraft Launcher

Лаунчер Minecraft-проекта [CubeXCraft](https://www.cubexcraft.site) для Windows, macOS и Linux:
вход в аккаунт, загрузка и проверка файлов игры, запуск, новости и голосование за проект, самообновление.
Tauri 2 (Rust) + Svelte 5.

**Скачивать лаунчер — только с https://www.cubexcraft.site.** Сборки из других мест не официальные.

Код открыт для просмотра, чтобы любой мог проверить, что лаунчер делает на компьютере игрока. Это не
open source: копировать, распространять и делать на его основе свои лаунчеры нельзя — см. [LICENSE](LICENSE).
Сторонние компоненты и их лицензии — [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Уязвимости — приватно, см. [SECURITY.md](SECURITY.md).

---

The launcher of the [CubeXCraft](https://www.cubexcraft.site) Minecraft project. Download it only from
https://www.cubexcraft.site. The source is published for review only and is **not** open source — see
[LICENSE](LICENSE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [SECURITY.md](SECURITY.md).

## Сборка для проверки / Building for review

```
npm ci
npm run tauri build -- --no-bundle
```

Требуются Node.js 20+ и Rust stable; для Linux — WebKitGTK 4.1.
