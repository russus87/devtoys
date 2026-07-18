<div align="center">

# DevToys

**A beautiful offline Swiss-army knife for developers.**

Every little tool you keep googling for — JSON/YAML converters, JWT decoder,
hashes, Base64, regex tester, formatters, generators and more — in one elegant
desktop app. Nothing ever leaves your machine.

Rust · Tauri 2 · Svelte 5

</div>

---

## Why

Most "dev toolbox" sites live in the browser, behind ads, and quietly send your
tokens and payloads to a server. DevToys does the same job **fully offline**, in
a native desktop window, so you can safely paste a production JWT or a customer
payload and know it stays on your computer.

## Tools

| Category | Tools |
| --- | --- |
| **Convertitori** | JSON ⇄ YAML · JSON ⇄ CSV · Analizza URL · Convertitore di base · Timestamp Unix · Convertitore colore · Contrasto colori (WCAG) · Numeri romani · Cron in italiano |
| **Codifica / Decodifica** | Base64 testo · Base64 ⇄ immagine · URL encode/decode · Entità HTML · Decoder JWT · Testo ⇄ Unicode · Basic Auth |
| **Formattatori** | JSON · SQL (multi-dialetto) · XML |
| **Generatori** | UUID (v4/v7) · Hash (MD5/SHA-1/2/3, CRC32) · HMAC · Password & token · Lorem ipsum · QR code |
| **Testo** | Convertitore maiuscole · Confronto testo (diff) · Statistiche testo · Tester regex · Anteprima Markdown · Escape stringa · Utilità righe · Slugify |
| **PDF** | PDF→immagini/testo/Word · Immagini→PDF · unisci/dividi/ruota/elimina/riordina · confronta · ottimizza/comprimi · sanitizza · filigrana · metadati · struttura · check PDF/UA · check PDF/A |
| **API & Test** | Load / performance test · Mock server · Generatore dati (IBAN/CF/P.IVA) · Smoke / synthetic runner · HTTP proxy / inspector · Codici di stato HTTP |

Text transforms run entirely in the frontend; cryptographic hashes, HMAC and
UUID generation are native Rust. The **PDF** tools reuse the Tauri-agnostic core
of PDF Accessibility Studio (via a pinned git dependency) plus the native Pdfium
library, bundled per-OS. The **API & Test** tools run all HTTP work in Rust
(tokio + reqwest + axum) — no CORS limits, and everything stays local. Note that
the mock server and proxy open a local listening port while active.

## Tray & auto-update

DevToys lives in the **system tray**: closing the window hides it there instead
of quitting. A **quick-launch popup** — a frameless, Spotlight-style window with
a searchable grid of every tool — opens on **`Ctrl/⌘ + Shift + Space`** (global
shortcut), on tray left-click (Windows/macOS), or from the tray menu (**⚡ Pannello
rapido**). Pick a tool and it opens in the main window; the popup dismisses itself
on `Esc` or when it loses focus. The tray menu also has **Apri DevToys**,
**Controlla aggiornamenti** and **Esci** (the way to fully quit).

> On tiling Wayland compositors (e.g. niri/Hyprland) the popup may need a window
> rule to float — match the title `DevToys — Pannello rapido`.

The app also **updates itself** from GitHub Releases, the same way
`russus_launcher` does: on startup (and every 6 h) it checks `russus87/devtoys`
for a newer version and, if found, shows a banner. **Scarica e installa** fetches
the right package for your platform and installs it — `pkexec pacman -U` on Arch,
elevated msiexec/NSIS on Windows, mount-and-copy on macOS — or **Apri release**
opens the download page. Everything stays offline until you ask it to check.

## Develop

```bash
npm install
npm run tauri dev      # run the app
npm run check          # svelte-check (type checking)
npm run tauri build    # build native bundles
```

Requires Rust (stable), Node 20+, and the usual Tauri Linux deps
(`webkit2gtk-4.1`, `gtk3`, `libappindicator-gtk3`).

## Releases

Pushing a `v*` tag builds and publishes, via GitHub Actions:

- **Arch Linux** — `.pkg.tar.zst`
- **Linux** — `.AppImage` and `.deb`
- **Windows** — `.msi` and NSIS `-setup.exe`
- **macOS** — `.dmg` (Apple Silicon + Intel)

## License

MIT © russus
