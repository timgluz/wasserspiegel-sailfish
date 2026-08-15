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
- The `[X-Sailjail] Permissions` value is **semicolon-separated**
  (e.g. `Permissions=Internet;Location`), and the permission name is the
  `.permission` filename (`Location` -> shown as "Positioning" in Settings).
- Allowed APIs: https://docs.sailfishos.org/Develop/Apps/Harbour/Allowed_APIs
- Allowed permissions: https://docs.sailfishos.org/Develop/Apps/Harbour/Allowed_Permissions

## Signing

```sh
gpg --gen-key                                   # create an RSA key (see below)
task sign:setup SIGNING_USER="Timo Sulg (Harbour signing)"
task sign                                       # sfdk build --prepare --sign
task verify                                     # rpm -K
task release                                    # sign + verify + sha256 (one-shot)
```

**The signing key must be RSA** — the SDK build engine ships `gpg2` 2.0.4
(2009), which cannot use modern `ed25519` keys (`gpg: Ohhhh jeeee: mpi
larger than packet`). Generate an RSA key:

```sh
gpg --full-generate-key   # choose "RSA and RSA", 3072 bits, add a comment
```

**The key must live in the build engine** (signing runs inside the engine,
which has its own gpg keyring, not the host's). Import it once:

```sh
gpg --export-secret-keys --armor "Timo Sulg (Harbour signing)" > signing-key.asc
docker cp signing-key.asc sailfish-sdk-build-engine_timgluz:/home/mersdk/
sfdk engine exec sh -c 'gpg2 --batch --import /home/mersdk/signing-key.asc && rm /home/mersdk/signing-key.asc'
```

First-time verification may report `digests SIGNATURES NOT OK`; import the
public key into the rpm keyring:

```sh
gpg --export --armor "Timo Sulg (Harbour signing)" > keyfile.gpg
sudo rpm --import keyfile.gpg
rpm -K RPMS/harbour-wasserspiegel-*.rpm          # digests signatures OK
```

See https://docs.sailfishos.org/Develop/Apps/Packaging/Signing_Packages
