# NOTICE:
#
# Application name defined in TARGET has a corresponding QML filename.
# If name defined in TARGET is changed, the following needs to be done
# to match new name:
#   - corresponding QML filename must be changed
#   - desktop icon filename must be changed
#   - desktop filename must be changed
#   - icon definition filename in desktop file must be changed
#   - translation filenames have to be changed

# The name of your application
TARGET = harbour-wasserspiegel

CONFIG += sailfishapp c++17
CONFIG += exceptions
QT += concurrent positioning

SOURCES += src/wasserspiegel.cpp \
    src/appcontroller.cpp

HEADERS += src/appcontroller.h

DISTFILES += qml/harbour-wasserspiegel.qml \
    qml/cover/CoverPage.qml \
    qml/pages/DashboardPage.qml \
    qml/pages/StationPickerPage.qml \
    qml/pages/SettingsPage.qml \
    qml/pages/AboutPage.qml \
    qml/pages/LogsPage.qml \
    qml/pages/TrendGraph.qml \
    rpm/wasserspiegel.changes.in \
    rpm/wasserspiegel.changes.run.in \
    rpm/wasserspiegel.spec \
    translations/*.ts \
    harbour-wasserspiegel.desktop

SAILFISHAPP_ICONS = 86x86 108x108 128x128 172x172

# Build-time API defaults (override in Settings at runtime).
# Sourced from the environment at qmake time, e.g. via direnv/.envrc.
WS_DEFAULT_API_BASE = $$(WASSERSPIEGEL_API)
WS_DEFAULT_API_TOKEN = $$(WASSERSPIEGEL_TOKEN)
!isEmpty(WS_DEFAULT_API_BASE) {
    DEFINES += WASSERSPIEGEL_DEFAULT_API_BASE=\\\"$$WS_DEFAULT_API_BASE\\\"
}
!isEmpty(WS_DEFAULT_API_TOKEN) {
    DEFINES += WASSERSPIEGEL_DEFAULT_API_TOKEN=\\\"$$WS_DEFAULT_API_TOKEN\\\"
}

# ---- Rust core (wasserspiegel-core staticlib, prebuilt) ----
#
# The staticlib is cross-compiled in the SDK build engine by
# `task engine:rust` (rust/engine-build.sh), which writes it to
# rust/target/<triple>/release/ and regenerates rust/include/.
# See README.md - the Rust toolchain cannot run under sb2, so cargo
# is intentionally NOT invoked from qmake.

RUST_DIR = $$absolute_path($$PWD/rust)

CARGO_TRIPLE = $$(CARGO_TARGET_TRIPLE)
isEmpty(CARGO_TRIPLE) {
    contains(QMAKE_CXX, .*aarch64.*) {
        CARGO_TRIPLE = aarch64-unknown-linux-gnu
    } else:contains(QMAKE_CXX, .*armv7.*) {
        CARGO_TRIPLE = armv7-unknown-linux-gnueabihf
    } else {
        CARGO_TRIPLE = aarch64-unknown-linux-gnu
    }
}

RUST_INCLUDE = $$RUST_DIR/include
RUST_LIB = $$RUST_DIR/target/$$CARGO_TRIPLE/release/libwasserspiegel_core.a

INCLUDEPATH += $$RUST_INCLUDE

exists($$RUST_LIB) {
    LIBS += $$RUST_LIB -ldl -lpthread -lrt -lm
} else {
    error("$$RUST_LIB not found - run 'task engine:rust' first (see README.md)")
}

# to disable building translations every time, comment out
# the following CONFIG line
CONFIG += sailfishapp_i18n

# German translation is enabled as an example. If you aren't
# planning to localize your app, remember to comment out
# the following TRANSLATIONS line. And also do not forget to
# modify the localized app name in the the desktop file.
TRANSLATIONS += translations/harbour-wasserspiegel-de.ts
