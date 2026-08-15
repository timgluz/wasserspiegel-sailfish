#include "appcontroller.h"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QMutex>
#include <QMutexLocker>
#include <QPointer>
#include <QStandardPaths>
#include <QTextStream>
#include <QtConcurrent>
#include <QTimer>

#include <cmath>
#include <cstdio>
#include <stdexcept>

#ifndef WASSERSPIEGEL_DEFAULT_API_BASE
#define WASSERSPIEGEL_DEFAULT_API_BASE ""
#endif
#ifndef WASSERSPIEGEL_DEFAULT_API_TOKEN
#define WASSERSPIEGEL_DEFAULT_API_TOKEN ""
#endif

namespace {

QString rustString(const rust::String &s)
{
    return QString::fromUtf8(s.data(), static_cast<int>(s.size()));
}

template <typename T, typename C>
rust::Slice<const T> constSlice(const C &c)
{
    return rust::Slice<const T>(c);
}

QString cacheDir()
{
    QString base = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    if (base.isEmpty())
        base = QStandardPaths::writableLocation(QStandardPaths::HomeLocation)
               + QStringLiteral("/.cache/harbour-wasserspiegel");
    return base + QStringLiteral("/cache");
}

// ---- in-memory log ring buffer (shown on the Logs page) ----

QMutex g_logMutex;
QStringList g_logLines;
const int g_maxLogLines = 400;

QString logLevelName(QtMsgType type)
{
    switch (type) {
    case QtDebugMsg: return QStringLiteral("D");
    case QtWarningMsg: return QStringLiteral("W");
    case QtCriticalMsg: return QStringLiteral("C");
    case QtFatalMsg: return QStringLiteral("F");
    case QtInfoMsg: return QStringLiteral("I");
    }
    return QStringLiteral("?");
}

void wsMessageHandler(QtMsgType type, const QMessageLogContext &ctx, const QString &msg)
{
    Q_UNUSED(ctx);
    const QString line = QStringLiteral("%1 %2 %3")
        .arg(QDateTime::currentDateTime().toString(QStringLiteral("HH:mm:ss.zzz")),
             logLevelName(type), msg);
    {
        QMutexLocker lock(&g_logMutex);
        g_logLines.append(line);
        while (g_logLines.size() > g_maxLogLines)
            g_logLines.removeFirst();
    }
    std::fprintf(stderr, "%s\n", line.toUtf8().constData());
    std::fflush(stderr);

    // also append to a file for on-device debugging (the Logs page shows
    // the in-memory buffer; the file survives crashes)
    static QFile f;
    if (!f.isOpen()) {
        const QString path = QStringLiteral("/home/defaultuser/.cache/wasserspiegel/debug.log");
        QDir().mkpath(QFileInfo(path).absolutePath());
        f.setFileName(path);
        f.open(QIODevice::WriteOnly | QIODevice::Append | QIODevice::Text);
    }
    if (f.isOpen()) {
        QTextStream ts(&f);
        ts << line << QLatin1Char('\n');
        ts.flush();
    }
}

// Errors from the Rust core that indicate bad/expired credentials or a
// missing API configuration - these should surface the Settings page.
bool isConfigError(const QString &msg)
{
    return msg.contains(QStringLiteral("authentication"))
        || msg.contains(QStringLiteral("configuration"))
        || msg.contains(QStringLiteral("not configured"));
}

} // namespace

void wsInstallMessageHandler()
{
    qInstallMessageHandler(wsMessageHandler);
}

AppController::AppController(QObject *parent)
    : QObject(parent)
    , m_settings(this)
{
}

AppController::~AppController() = default;

void AppController::initialize()
{
    m_apiBase = m_settings.value(
                    QStringLiteral("api/base"),
                    QString::fromUtf8(WASSERSPIEGEL_DEFAULT_API_BASE)).toString();
    m_apiToken = m_settings.value(
                    QStringLiteral("api/token"),
                    QString::fromUtf8(WASSERSPIEGEL_DEFAULT_API_TOKEN)).toString();

    m_graphPeriodHours = m_settings.value(
                             QStringLiteral("graph/periodHours"), 72).toInt();

    m_stationId = m_settings.value(QStringLiteral("station/id")).toString();
    m_stationName = m_settings.value(QStringLiteral("station/name")).toString();
    m_water = m_settings.value(QStringLiteral("station/water")).toString();

    createClient();
    setNeedsConfig(!configured());

    if (!m_stationId.isEmpty()) {
        loadCachedIntoView(m_stationId);
        refresh();
    } else {
        // first launch or no station yet: seed the UI with demo data so
        // it never looks broken; the user picks a real station later
        applyDemoData();
    }
    emit settingsChanged();
    emit stationChanged();
}

bool AppController::createClient()
{
    if (m_apiBase.isEmpty() || m_apiToken.isEmpty()) {
        m_clientReady = false;
        setNeedsConfig(true);
        return false;
    }
    try {
        auto box = wasserspiegel::new_client(
            m_apiBase.toStdString(), m_apiToken.toStdString(),
            cacheDir().toStdString());
        m_client = std::make_shared<ClientHolder>(ClientHolder{ std::move(box) });
        m_clientReady = true;
        return true;
    } catch (const rust::Error &e) {
        m_client.reset();
        m_clientReady = false;
        setError(QObject::tr("API configuration invalid: %1").arg(QString::fromUtf8(e.what())));
        setNeedsConfig(true);
        return false;
    }
}

// ---- refresh / selection ----

void AppController::refresh()
{
    if (m_loading || m_stationId.isEmpty() || !m_clientReady)
        return;

    m_loading = true;
    emit loadingChanged();

    const QString id = m_stationId;
    const auto clientHolder = m_client;

    const QPointer<AppController> guard(this);
    QtConcurrent::run([clientHolder, id, guard]() {
        auto *client = &*clientHolder->box;
        std::shared_ptr<wasserspiegel::FfiStationMetrics> metrics;
        QString error;
        try {
            metrics = std::make_shared<wasserspiegel::FfiStationMetrics>(
                client->fetch_station(id.toStdString()));
        } catch (const rust::Error &e) {
            error = QString::fromUtf8(e.what());
        } catch (const std::exception &e) {
            error = QString::fromUtf8(e.what());
        }

        if (guard.isNull())
            return;
        QTimer::singleShot(0, guard, [guard, metrics, error]() {
            guard->m_loading = false;
            emit guard->loadingChanged();
            if (metrics) {
                guard->setError(QString());
                guard->setNeedsConfig(false);
                guard->populateFromMetrics(*metrics, true);
            } else {
                guard->setError(error);
                // fall back to cache if we have nothing on screen
                if (!guard->m_hasData)
                    guard->loadCachedIntoView(guard->m_stationId);
            }
        });
    });
}

void AppController::selectStation(const QString &id, const QString &name, const QString &water)
{
    if (id.isEmpty())
        return;

    m_stationId = id;
    m_stationName = name;
    m_water = water;
    m_currentLevel = 0.0;
    m_hasData = false;
    m_fromCache = false;
    m_lastUpdatedMs = 0;
    m_seriesPoints.clear();

    m_settings.setValue(QStringLiteral("station/id"), id);
    m_settings.setValue(QStringLiteral("station/name"), name);
    m_settings.setValue(QStringLiteral("station/water"), water);

    emit stationChanged();
    emit graphSeriesChanged();

    loadCachedIntoView(id);
    refresh();
}

void AppController::loadCachedIntoView(const QString &stationId)
{
    if (!m_clientReady)
        return;
    try {
        const auto metrics = m_client->box->load_cached_station(stationId.toStdString());
        populateFromMetrics(metrics, false);
    } catch (const rust::Error &) {
        // no cache yet - nothing to do
    }
}

void AppController::populateFromMetrics(const wasserspiegel::FfiStationMetrics &m, bool persist)
{
    m_stationId = rustString(m.station_id);
    m_stationName = rustString(m.station_name);
    m_water = rustString(m.water);
    m_km = m.km;
    m_currentLevel = m.current_level;
    m_unit = rustString(m.unit);
    m_change1Day = m.change_1day;
    m_change3Day = m.change_3day;
    m_change7Day = m.change_7day;
    m_fromCache = m.from_cache;
    m_lastUpdatedMs = m.fetched_at_ms;

    m_history = m.history; // deep copy into our own rust::Vec
    m_hasData = true;

    // Only a fresh network payload rewrites the persisted station:
    // cached data may be stale and must not clobber explicit choices.
    if (persist) {
        m_settings.setValue(QStringLiteral("station/id"), m_stationId);
        m_settings.setValue(QStringLiteral("station/name"), m_stationName);
        m_settings.setValue(QStringLiteral("station/water"), m_water);
    }

    recomputeSeries();
    emit stationChanged();
}

// ---- graph series ----

void AppController::setGraphPeriodHours(int hours)
{
    if (hours < 1)
        hours = 1;
    if (hours == m_graphPeriodHours)
        return;
    m_graphPeriodHours = hours;
    m_settings.setValue(QStringLiteral("graph/periodHours"), hours);
    recomputeSeries();
    emit graphSeriesChanged();
}

void AppController::recomputeSeries()
{
    m_seriesPoints.clear();
    m_seriesMin = 0.0;
    m_seriesMax = 1.0;
    m_seriesStartMs = 0;
    m_seriesEndMs = 0;

    if (m_history.empty()) {
        return;
    }

    const auto sliced = wasserspiegel::slice_series(
        constSlice<wasserspiegel::FfiMeasurementPoint>(m_history),
        m_graphPeriodHours, 200);
    if (sliced.empty()) {
        return;
    }

    const auto range = wasserspiegel::series_range(
        constSlice<wasserspiegel::FfiMeasurementPoint>(sliced));
    m_seriesMin = range.min;
    m_seriesMax = range.max;

    const qint64 t0 = sliced.front().timestamp_ms;
    const qint64 t1 = sliced.back().timestamp_ms;
    const double span = qMax<qint64>(t1 - t0, 1);
    m_seriesStartMs = t0;
    m_seriesEndMs = t1;

    for (const auto &p : sliced) {
        QVariantMap point;
        point.insert(QStringLiteral("x"), (p.timestamp_ms - t0) / span);
        point.insert(QStringLiteral("y"), p.value);
        m_seriesPoints.append(point);
    }
}

// ---- station search ----

void AppController::searchStations(const QString &query)
{
    const QString q = query.trimmed();
    if (q.size() < 2) {
        m_searchResults.clear();
        emit searchResultsChanged();
        return;
    }

    if (!m_stationList.empty()) {
        const auto filtered = wasserspiegel::filter_stations(
            constSlice<wasserspiegel::FfiStationSummary>(m_stationList),
            q.toStdString(), 50);
        m_searchResults = summariesToVariantList(filtered);
        emit searchResultsChanged();
        return;
    }

    ensureStationListLoaded();
}

void AppController::ensureStationListLoaded()
{
    // 1) try cache synchronously
    if (m_clientReady) {
        try {
            const auto cached = m_client->box->load_cached_stations();
            m_stationList.assign(cached.begin(), cached.end());
            emit stationListReady();
            return;
        } catch (const rust::Error &) {
            // fall through to network fetch
        }
    }

    // 2) fetch asynchronously
    if (!m_clientReady || m_stationsLoading)
        return;

    m_stationsLoading = true;
    emit stationsLoadingChanged();

    const auto clientHolder = m_client;
    const QPointer<AppController> guard(this);
    QtConcurrent::run([clientHolder, guard]() {
        auto *client = &*clientHolder->box;
        std::shared_ptr<std::vector<wasserspiegel::FfiStationSummary>> list;
        QString error;
        try {
            auto v = std::make_shared<rust::Vec<wasserspiegel::FfiStationSummary>>(
                client->fetch_stations());
            list = std::make_shared<std::vector<wasserspiegel::FfiStationSummary>>(
                v->begin(), v->end());
        } catch (const rust::Error &e) {
            error = QString::fromUtf8(e.what());
        } catch (const std::exception &e) {
            error = QString::fromUtf8(e.what());
        }

        if (guard.isNull())
            return;
        QTimer::singleShot(0, guard, [guard, list, error]() {
            guard->m_stationsLoading = false;
            emit guard->stationsLoadingChanged();
            if (list) {
                guard->m_stationList = *list;
                emit guard->stationListReady();
            } else if (!error.isEmpty()) {
                guard->setError(error);
            }
        });
    });
}

QVariantList AppController::summariesToVariantList(
    const rust::Vec<wasserspiegel::FfiStationSummary> &list)
{
    QVariantList out;
    out.reserve(static_cast<int>(list.size()));
    for (const auto &s : list) {
        QVariantMap item;
        item.insert(QStringLiteral("id"), rustString(s.id));
        item.insert(QStringLiteral("name"), rustString(s.name));
        item.insert(QStringLiteral("water"), rustString(s.water));
        item.insert(QStringLiteral("km"), s.km);
        out.append(item);
    }
    return out;
}

// ---- settings ----

void AppController::setApiBase(const QString &value)
{
    if (value == m_apiBase)
        return;
    m_apiBase = value;
    m_settings.setValue(QStringLiteral("api/base"), value);
    createClient();
    emit settingsChanged();
}

void AppController::setApiToken(const QString &value)
{
    if (value == m_apiToken)
        return;
    m_apiToken = value;
    m_settings.setValue(QStringLiteral("api/token"), value);
    createClient();
    emit settingsChanged();
}

void AppController::applySettings(const QString &apiBase, const QString &apiToken)
{
    m_apiBase = apiBase.trimmed();
    m_apiToken = apiToken.trimmed();
    m_settings.setValue(QStringLiteral("api/base"), m_apiBase);
    m_settings.setValue(QStringLiteral("api/token"), m_apiToken);
    createClient();
    emit settingsChanged();
}

void AppController::testConnection()
{
    if (!m_clientReady) {
        emit connectionTested(false, tr("API URL or token is not configured"));
        return;
    }

    const auto clientHolder = m_client;
    const QPointer<AppController> guard(this);
    QtConcurrent::run([clientHolder, guard]() {
        auto *client = &*clientHolder->box;
        bool ok = false;
        QString message;
        std::shared_ptr<std::vector<wasserspiegel::FfiStationSummary>> list;
        try {
            const auto stations = client->fetch_stations();
            ok = true;
            message = AppController::tr("OK - %1 stations loaded")
                          .arg(static_cast<int>(stations.size()));
            list = std::make_shared<std::vector<wasserspiegel::FfiStationSummary>>(
                stations.begin(), stations.end());
        } catch (const rust::Error &e) {
            message = QString::fromUtf8(e.what());
        } catch (const std::exception &e) {
            message = QString::fromUtf8(e.what());
        }

        if (guard.isNull())
            return;
        QTimer::singleShot(0, guard, [guard, ok, message, list]() {
            if (list) {
                // reuse the fetched list for the station picker
                guard->m_stationList = *list;
                emit guard->stationListReady();
            }
            if (ok) {
                guard->setError(QString());
                guard->setNeedsConfig(false);
                // credentials work now - refresh the dashboard if a station is set
                if (!guard->m_stationId.isEmpty())
                    guard->refresh();
            }
            emit guard->connectionTested(ok, message);
        });
    });
}

void AppController::setError(const QString &message)
{
    if (message == m_error)
        return;
    m_error = message;
    if (!message.isEmpty() && isConfigError(message))
        setNeedsConfig(true);
    emit errorChanged();
}

void AppController::setNeedsConfig(bool value)
{
    if (value == m_needsConfig)
        return;
    m_needsConfig = value;
    emit needsConfigChanged();
}

void AppController::applyDemoData()
{
    m_stationId.clear();
    m_stationName = QStringLiteral("Demo station");
    m_water = QStringLiteral("Rhein");
    m_km = 424.7;
    m_currentLevel = 88.0;
    m_unit = QStringLiteral("cm");
    m_change1Day = 3.2;
    m_change3Day = -1.8;
    m_change7Day = 16.4;
    m_fromCache = false;
    m_lastUpdatedMs = 0;
    m_hasData = true;

    rust::Vec<wasserspiegel::FfiMeasurementPoint> history;
    const qint64 now = QDateTime::currentMSecsSinceEpoch();
    const int points = 288; // ~3 days at 15-minute intervals
    for (int i = 0; i < points; ++i) {
        wasserspiegel::FfiMeasurementPoint p;
        p.timestamp_ms = now - qint64(points - i) * 15 * 60 * 1000LL;
        p.value = 82.0 + 10.0 * std::sin(i / 16.0) + 3.0 * std::sin(i / 5.0);
        history.push_back(p);
    }
    m_history = history;

    recomputeSeries();
    emit stationChanged();
}

QString AppController::logText() const
{
    QMutexLocker lock(&g_logMutex);
    return g_logLines.join(QLatin1Char('\n'));
}
