# Harbour packaging notes

Reference for Sailfish OS (Harbour / Jolla Store) package naming and
quality requirements.

## RPM file naming

The file name must be:

    harbour-NAME-VERSION-RELEASE.ARCHITECTURE.rpm

- Max length: 100 characters.
- `NAME`: lowercase letters (`a-z`), digits (`0-9`), dashes (`-`),
  underscores (`_`), periods (`.`). 3-50 characters, must start with an
  alphanumeric character. Must be prefixed `harbour-`.
- `VERSION`: `major[.minor][.patch]`, digits only. On update:
  `major` may only increase; `minor` may decrease only if `major` increased;
  `patch` may decrease only if `major` or `minor` increased.
- `RELEASE`: digits (`0-9`), underscores (`_`), periods (`.`).
- `ARCHITECTURE`: `aarch64`, `armv7hl`, `i486` or `noarch`.

### App identity checklist (when renaming to `harbour-...`)

| What | Where |
| --- | --- |
| package name | `rpm/wasserspiegel.spec` → `Name: harbour-wasserspiegel` |
| binary name | `wasserspiegel.pro` → `TARGET = harbour-wasserspiegel` |
| main QML | `qml/harbour-wasserspiegel.qml` |
| desktop file | `harbour-wasserspiegel.desktop` (`Exec`, `Icon`, `Name`, `[X-Sailjail] ApplicationName`) |
| icons | `icons/<size>/harbour-wasserspiegel.png` |
| translations | `translations/harbour-wasserspiegel-*.ts` |
| runtime org/app | `setApplicationName("harbour-wasserspiegel")` (must match the desktop `ApplicationName` or sailjail persistence breaks) |

## Validating

```sh
task check          # sfdk check (Harbour validator + rpmlint)
```

- Runs automatically during `sfdk build` too (skip with `--no-check`).
- Suites: `harbour` (allowed APIs/permissions), `rpmlint`, `rpmspec`.
- Harbour **disallows** `QtPositioning` import and the `Location` sailjail
  permission -> the GPS/"find nearest" feature is not Harbour-clean.
- Allowed APIs: https://docs.sailfishos.org/Develop/Apps/Harbour/Allowed_APIs
- Allowed permissions: https://docs.sailfishos.org/Develop/Apps/Harbour/Allowed_Permissions

## Signing

```sh
gpg --gen-key
task sign:setup SIGNING_USER="Timo Sulg"          # + optional PASSPHRASE_FILE
task sign                                         # sfdk build --prepare --sign
task verify                                       # rpm -K
```

First-time verification may report `digests SIGNATURES NOT OK`; import the
key into the rpm keyring:

```sh
gpg --output keyfile.gpg --armor --export "Timo Sulg"
rpm --import keyfile.gpg
```

See https://docs.sailfishos.org/Develop/Apps/Packaging/Signing_Packages
