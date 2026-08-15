# Wasserspiegel - Sailfish OS water level dashboard

Shows the current water level, trend deltas (24 h / 3 d / 7 d) and a level
history graph for a PegelOnline measurement station (e.g. Mannheim / Rhein),
plus an active cover with the current reading. Works offline from cache;
the last selected station is remembered.

Data comes from the wasserspiegel API (the Fermyon Spin service behind the
TRMNL plugin): `GET /stations` for the station list and
`GET /stations/{uuid}` for live dashboard data.

## Architecture

- `rust/` - **wasserspiegel-core**: everything non-UI lives in Rust
  (ureq + rustls HTTPS client, serde models, JSON file cache, graph
  slicing/downsampling). Exposed to C++ via a `cxx` bridge; the generated
  header lands in `rust/include/` at build time.
- `src/` - thin C++ glue: `AppController` (QObject) runs blocking bridge
  calls on `QtConcurrent` workers, exposes QML properties and persists
  settings / last station via QSettings.
- `qml/` - Silica UI: dashboard with Canvas trend graph, station picker
  (client-side search), settings page, active cover.

## Getting started

### 1. Prerequisites (host)

- [Sailfish OS SDK](https://sailfishos.org/develop/) (`sfdk` on your PATH;
  the installer does not add it automatically):

  ```sh
  export PATH="${HOME}/SailfishOS/bin:$PATH"   # add to ~/.zshrc / ~/.bashrc
  ```
- [Task](https://taskfile.dev) 3.x (`task` binary) - optional but handy
- Network access; secrets in `.envrc` (untracked). Copy `env.example` or
  just create it - both the build and `task deploy` read from here:

  ```sh
  # source this in every shell (or `direnv allow .`)
  export WASSERSPIEGEL_API="https://wasserspiegel-hnhptksa.fermyon.app"
  export WASSERSPIEGEL_TOKEN="..."                 # API bearer token
  export JOLLA_SSH_PASSWORD="..."                  # phone developer password
                                                    # (Developer tools -> Remote connection)
  ```

  ```sh
  source .envrc    # or: direnv allow .
  ```

Before the first build, point sfdk at the installed SDK target:

```sh
sfdk config --global target=SailfishOS-5.1.0.11-aarch64   # pick your SDK version
```

### 2. Install Rust into the SDK build engine (once per SDK)

`sfdk build` compiles inside the SDK build engine (a container), so the
Rust toolchain must live there - a host toolchain is not enough.

```sh
task engine:setup
```

This (via `rust/engine-setup.sh`) installs, inside the engine:

- rustup as the `mersdk` user with the **i686** host toolchain (the engine
  is 32-bit) and the `aarch64-unknown-linux-gnu` target
- host `gcc-c++` (needed by the `link-cplusplus` build script)
- cross-toolchain wrappers adding the correct `--sysroot` and `as`/`ld`
  (the SDK's `aarch64-meego-linux-gnu-g++` ships no usable default sysroot)

Re-run `task engine:setup` (it is idempotent) if you ever reset the engine.

> Why not plain `su`/`sudo` + rustup? The engine's root password is locked
> (`su` fails for any password), but `sudo` is passwordless for `mersdk`.
> rustup must be installed as `mersdk` - its home is host-mapped, so sb2
> can see the toolchain - and as the i686 host, since the engine has a
> 32-bit userspace.

### 3. Build the Rust core (dependencies) for aarch64

The Rust staticlib is cross-compiled **in the engine**, because the SDK
cross toolchain (and its sysroot) only exist there. cargo must run *outside*
sb2 (sb2 hangs host tools), so this is a dedicated step:

```sh
task engine:rust
```

This (via `rust/engine-build.sh`) stages `rust/` into the engine, runs
`cargo build --release --target aarch64-unknown-linux-gnu` with the
cross-toolchain wrappers, and copies the result back:

- `rust/target/aarch64-unknown-linux-gnu/release/libwasserspiegel_core.a`
- `rust/include/` (the cxx bridge headers)

The qmake step links these prebuilt artifacts - cargo is intentionally
**not** invoked from qmake (it cannot run under sb2).

### 4. Sanity-check the Rust core (host, no SDK needed)

```sh
task test        # unit + fixture tests
task test:all    # + live API tests + host C++ bridge smoke test (needs .envrc)
```

### 5. Build the RPM

With `.envrc` sourced (so `WASSERSPIEGEL_API` / `WASSERSPIEGEL_TOKEN` get
baked in as build-time defaults - optional, everything is also configurable
at runtime in Settings):

```sh
direnv allow .        # or: source .envrc
task build            # = task engine:rust + sfdk build
```

Output lands in `RPMS/wasserspiegel-*.rpm`.

Notes:

- The token baked at build time is a convenience default for personal
  builds. Do **not** ship packages built with it - build without the env
  vars and let users configure Settings instead.
- `task build` re-runs the Rust cross-build every time; to iterate only on
  the C++/QML side use `sfdk build` directly (the prebuilt `.a` is already
  there).

### 6. Prepare the phone

1. Settings -> Developer tools -> enable **Developer mode** and
   **Remote connection (SSH)**; set a password and note the IP shown
   (e.g. `192.168.2.15`). The SSH user is `defaultuser` on Sailfish OS
   4.4+ (older releases use `nemo`).
2. Put that developer password in `.envrc` as `JOLLA_SSH_PASSWORD`
   (see step 1) - `task deploy` uses it to run `devel-su`.
3. Make sure the phone and your host are on the same network (or use USB
   networking).

### 7. Install and run

```sh
source .envrc                        # needed for JOLLA_SSH_PASSWORD
task deploy                          # uses the default DEVICE from Taskfile.yml
task deploy DEVICE=nemo@10.0.0.2     # one-off override
```

which is equivalent to:

```sh
scp RPMS/wasserspiegel-*.rpm defaultuser@192.168.2.15:/tmp/wasserspiegel.rpm
ssh defaultuser@192.168.2.15 "echo '$JOLLA_SSH_PASSWORD' | devel-su pkcon install-local -y /tmp/wasserspiegel.rpm"
```

(The RPM is staged in `/tmp/` on the phone, so the user's home dir doesn't
matter - `defaultuser` vs `nemo`. `pkcon install-local` reinstalls the same
version, so re-deploying after a rebuild works.)

Re-running the same command upgrades the installed package (same version,
new build). Then launch **Wasserspiegel** from the app grid.

### 8. First run

1. If API defaults were baked in, the app opens straight on the station
   picker; otherwise open Settings, enter API URL + token and tap
   **Save & test**.
2. Pick a station (search by name or river, e.g. `mannheim rhein` - note
   there is both MANNHEIM / NECKAR and MANNHEIM / RHEIN).
3. The dashboard shows the current level, deltas and the trend graph
   (24 h / 3 d / 10 d). Pull down for Refresh / Change station / Settings.
4. Add the app to the ambience/home screen cover to see the level at a
   glance; the cover sync button refreshes without opening the app.

### Updating & logs

```sh
task build && task deploy
ssh defaultuser@192.168.2.15   # then: journalctl --user -f | grep -i wasserspiegel
```

## Taskfile quick reference

| Task | What it does |
| --- | --- |
| `task engine:setup` | one-time Rust toolchain setup in the SDK engine |
| `task engine:rust` | cross-compile the Rust core for aarch64 in the engine |
| `task build` | `engine:rust` + `sfdk build` |
| `task deploy` | scp + `pkcon install-local` (default `defaultuser@192.168.2.15`) |
| `task test` / `task test:live` / `task smoke` | Rust tests / live API / host bridge smoke |
| `task test:all` | all of the above |
| `task lint` / `task fmt` | clippy + rustfmt on the Rust core |

## Troubleshooting

- **`failed to connect to the docker API at unix:///var/run/docker.sock` /
  sfdk hangs on "Starting the build engine"** - the Docker daemon isn't
  running (sfdk's socket check does not trigger socket activation):

  ```sh
  sudo systemctl start docker   # once; enabled => auto-starts on reboot
  ```

  A hanging sfdk ignores the first `Ctrl+C` (it traps signals for
  engine cleanup) - use `Ctrl+\` or `pkill -9 -f sfdk` from another
  terminal.

- **`su: Authentication failure` in the engine** - the engine's root
  password is locked; use `sfdk engine exec sudo ...` (passwordless for
  `mersdk`) instead of `su`.
- **`libatomic.so.1: cannot open shared object file`** - install the
  engine's `libatomic` package: `sfdk engine exec sudo zypper --non-interactive install libatomic`.
- **`.pro` errors: `libwasserspiegel_core.a not found`** - the Rust core
  wasn't cross-built; run `task engine:rust` (or `task build`).
- **First search shows nothing / spinner** - the full station list (787
  entries) is fetched once and cached; the pull-down "Reload station list"
  retries. Offline installs start from the cache only after the first
  successful fetch.
- **Auth error on refresh** - check the token in Settings; the API sends
  `401` on a bad token.
- **App not in the app grid after install** - run
  `ssh ... "devel-su pkcon install-local -y /tmp/wasserspiegel.rpm"`
  again and watch for errors; a stale older version may need
  `devel-su pkcon remove wasserspiegel` first.

## API quirks handled by the client

- `GET /stations` returns all ~787 stations at once, raw PegelOnline
  shape (`uuid`, `longname`, `water.longname`), pagination params ignored.
- `/search/stations` is currently broken server-side -> search runs
  client-side over the cached station list.
- Unknown station ids come back as HTTP 500 with a JSON error envelope;
  the client maps the body to proper errors (auth / not found / server).

## Roadmap

- GPS-based nearest station suggestion
- Background cover refresh
- Multiple favourites
