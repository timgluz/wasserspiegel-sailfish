# Wasserspiegel — session notes / follow-ups

## Current state (v0.2-1)

- Harbour-clean build: package `harbour-wasserspiegel`, validation passes
  (`sfdk check` → 0 errors), signed RPM produced and deployed to the phone.
- Features: station search (city/river) + recent stations, live dashboard
  with trend graph (24 h/3 d/10 d), demo-data fallback, config flow,
  Logs page, About page, active cover.
- Signed RPM: `RPMS/harbour-wasserspiegel-0.2-1.aarch64.rpm` (verified OK).

## Follow-up tasks

1. **GPS "find nearest station"** — deliberately removed for Harbour
   (QtPositioning import + `Location` permission are disallowed).
   The compute logic is still in place (Rust `nearest_station`,
   C++ `findNearestStation`/`doFindNearest`); re-add only:
   - position acquisition (QML `PositionSource` *or* C++ `QGeoPositionInfoSource`)
   - the picker button + `Location` sailjail permission + `QT += positioning`
2. **On-disk `debug.log`** — still not written under sailjail; in-app Logs
   page is the working alternative.
3. **"file is not stripped" warning** — cosmetic; strip the binary if wanted.

## Build/deploy gotchas (already solved, keep in mind)

- `task build` uses `sfdk build --prepare` (needs a **clean git tree** —
  commit first). `sfdk build` without `--prepare` uses a stale source snapshot.
- Sailfish sailjail `Permissions` is **semicolon-separated**
  (`Permissions=Internet`), and the desktop `OrganizationName` must match
  the app's runtime org (`org.timgluz`) or cache/settings persistence breaks.
- Signing key must be **RSA** (SDK engine ships `gpg2` 2.0.4, no ed25519),
  and must be imported into the **build engine's** keyring. See
  `notes/harbour-packaging.md`.
- Translations `translations/*.ts` get rewritten by `lupdate` on every build
  (dirty tree) — commit them before `--prepare`.

## Quick commands

```sh
task build && task deploy      # build + install on phone
task screenshots               # pull + upscale store screenshots
task check                     # Harbour validator + rpmlint
task release                   # sign + verify + sha256
```
