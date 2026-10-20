# pikiosk-apps

The app store for [pikiosk](https://github.com/pierrejochem/pikiosk): one folder per app, a workflow that
builds and signs them, and the store itself, a static [TUF](https://theupdateframework.io) repository on
the `gh-pages` branch.

```
apps/<name>/              a Rust crate on kiosk-kit, plus package/manifest.toml and package/assets/
scripts/init-keys.sh      one-time key ceremony (offline)
scripts/build-app.sh      cross-build an app against the pikiosk SDK and pack it as .kpkg
scripts/publish.sh        sign a channel: catalog.json, targets, snapshot, timestamp
scripts/live-packages.sh  the packages already published in a channel
root.json                 the trust anchor devices are built with
.github/workflows/        publish.yml
```

## How a device trusts this

A pikiosk image carries `root.json` (`package/kioskd/root.json` in the pikiosk repo). Only repositories
signed by the keys listed there are believed. Metadata expires, only ever moves forward (an older but
validly signed repository is refused), and every package is checked against the length and SHA-256 in
the signed metadata before it is unpacked. Set `STORE_URL=https://<user>.github.io/pikiosk-apps` and
`STORE_CHANNEL=stable` (or `beta`) in `kiosk.conf` to turn it on.

> `root.json` here is the same file as `package/kioskd/root.json` in the pikiosk repo. The four private keys are not in any repository; keep `root.der` offline.

## Key ceremony (once, offline)

```sh
cargo install tuftool --locked
scripts/init-keys.sh ~/pikiosk-keys root.json
```

- Keep `root.der` offline. It is only needed to rotate keys.
- Store `targets.der`, `snapshot.der` and `timestamp.der` as base64 in the GitHub Actions secrets
  `TARGETS_KEY`, `SNAPSHOT_KEY` and `TIMESTAMP_KEY` (`base64 -w0 targets.der`).
- Copy `root.json` to `package/kioskd/root.json` in the pikiosk repo.
- Set the repository variable `PIKIOSK_TAG` to the pikiosk release whose SDK and `kpkg` the workflow uses.
- Enable GitHub Pages for the `gh-pages` branch.

## Publishing

1. Tag `hello-v1.0.0` (the app folder name, `-v`, the manifest version). The workflow builds
   `apps/hello` in the pikiosk SDK and publishes it to the **beta** channel.
2. Run the workflow with `promote-stable` to copy everything in beta to **stable**.
3. A daily run re-signs both channels so the timestamp never expires on a device.

The repository layout is `<channel>/metadata/` and `<channel>/targets/`. Metadata versions are
computed from the live repository and only ever increase.

## Writing an app

Start from `apps/hello`. An app is a Rust program that implements `kiosk_kit::App` and calls
`kiosk_kit::run`. `package/manifest.toml` names it (`id`, `version`, `name`, `category`, `entry`,
`icon`, `boards`, `permissions`); `kpkg build` checks it and the 128×128 PNG icon.

Build and try it on your machine (needs the pikiosk repo next to this one):

```sh
cd apps/hello
cargo build --release --config 'patch."https://github.com/pierrejochem/pikiosk".kiosk-kit.path="../../../pikiosk/kiosk-kit"'
```

## License

MIT, see [LICENSE](LICENSE).
