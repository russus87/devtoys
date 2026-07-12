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
| **Convertitori** | JSON ⇄ YAML · Convertitore di base · Timestamp Unix · Convertitore colore · Cron in italiano |
| **Codifica / Decodifica** | Base64 testo · Base64 ⇄ immagine · URL encode/decode · Entità HTML · Decoder JWT |
| **Formattatori** | JSON · SQL (multi-dialetto) · XML |
| **Generatori** | UUID (v4/v7) · Hash (MD5/SHA-1/2/3, CRC32) · HMAC · Password & token · Lorem ipsum · QR code |
| **Testo** | Convertitore maiuscole · Confronto testo (diff) · Statistiche testo · Tester regex · Anteprima Markdown · Escape stringa |

Text transforms run entirely in the frontend; cryptographic hashes, HMAC and
UUID generation are done natively in Rust.

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
