#ifndef APPCONTROLLER_H
#define APPCONTROLLER_H

#include <QObject>
#include <QSettings>
#include <QVariantList>
#include <memory>
#include <vector>

#include "wasserspiegel_core.h"

// rust::Box is move-only and has no default constructor, so we hold it in
// a small wrapper. AppController owns it via shared_ptr so in-flight
// QtConcurrent workers keep the old client alive while settings changes
// swap in a new one.
struct ClientHolder
{
    rust::Box<wasserspiegel::CoreClient> box;
};

class AppController : public QObject
{
    Q_OBJECT

    // currently selected station + dashboard values
    Q_PROPERTY(QString stationId READ stationId NOTIFY stationChanged)
    Q_PROPERTY(bool hasStation READ hasStation NOTIFY stationChanged)
    Q_PROPERTY(QString stationName READ stationName NOTIFY stationChanged)
    Q_PROPERTY(QString water READ water NOTIFY stationChanged)
    Q_PROPERTY(double km READ km NOTIFY stationChanged)
    Q_PROPERTY(double currentLevel READ currentLevel NOTIFY stationChanged)
    Q_PROPERTY(QString unit READ unit NOTIFY stationChanged)
    Q_PROPERTY(double change1Day READ change1Day NOTIFY stationChanged)
    Q_PROPERTY(double change3Day READ change3Day NOTIFY stationChanged)
    Q_PROPERTY(double change7Day READ change7Day NOTIFY stationChanged)
    Q_PROPERTY(bool fromCache READ fromCache NOTIFY stationChanged)
    Q_PROPERTY(qint64 lastUpdatedMs READ lastUpdatedMs NOTIFY stationChanged)

    // network state
    Q_PROPERTY(bool loading READ loading NOTIFY loadingChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

    // graph series (recomputed when period or data changes)
    Q_PROPERTY(int graphPeriodHours READ graphPeriodHours WRITE setGraphPeriodHours NOTIFY graphSeriesChanged)
    Q_PROPERTY(QVariantList seriesPoints READ seriesPoints NOTIFY graphSeriesChanged)
    Q_PROPERTY(double seriesMin READ seriesMin NOTIFY graphSeriesChanged)
    Q_PROPERTY(double seriesMax READ seriesMax NOTIFY graphSeriesChanged)
    Q_PROPERTY(qint64 seriesStartMs READ seriesStartMs NOTIFY graphSeriesChanged)
    Q_PROPERTY(qint64 seriesEndMs READ seriesEndMs NOTIFY graphSeriesChanged)

    // station search
    Q_PROPERTY(QVariantList searchResults READ searchResults NOTIFY searchResultsChanged)
    Q_PROPERTY(bool stationsLoading READ stationsLoading NOTIFY stationsLoadingChanged)
    Q_PROPERTY(QVariantList recentStations READ recentStations NOTIFY recentStationsChanged)

    // settings (QSettings-backed, applied to the client immediately)
    Q_PROPERTY(QString apiBase READ apiBase WRITE setApiBase NOTIFY settingsChanged)
    Q_PROPERTY(QString apiToken READ apiToken WRITE setApiToken NOTIFY settingsChanged)
    Q_PROPERTY(bool configured READ configured NOTIFY settingsChanged)

    // set when the API is not configured or a fetch failed due to
    // bad/expired credentials - the UI offers the Settings page then
    Q_PROPERTY(bool needsConfig READ needsConfig NOTIFY needsConfigChanged)

public:
    explicit AppController(QObject *parent = nullptr);
    ~AppController() override;

    void initialize();

    Q_INVOKABLE void refresh();
    Q_INVOKABLE void selectStation(const QString &id, const QString &name, const QString &water);
    Q_INVOKABLE void searchStations(const QString &query);
    Q_INVOKABLE void applySettings(const QString &apiBase, const QString &apiToken);
    Q_INVOKABLE void testConnection();
    Q_INVOKABLE QString logText() const;
    Q_INVOKABLE void findNearestStation(double lat, double lon);

    // property getters
    QString stationId() const { return m_stationId; }
    bool hasStation() const { return !m_stationId.isEmpty(); }
    QString stationName() const { return m_stationName; }
    QString water() const { return m_water; }
    double km() const { return m_km; }
    double currentLevel() const { return m_currentLevel; }
    QString unit() const { return m_unit; }
    double change1Day() const { return m_change1Day; }
    double change3Day() const { return m_change3Day; }
    double change7Day() const { return m_change7Day; }
    bool fromCache() const { return m_fromCache; }
    qint64 lastUpdatedMs() const { return m_lastUpdatedMs; }
    bool loading() const { return m_loading; }
    QString error() const { return m_error; }
    int graphPeriodHours() const { return m_graphPeriodHours; }
    QVariantList seriesPoints() const { return m_seriesPoints; }
    double seriesMin() const { return m_seriesMin; }
    double seriesMax() const { return m_seriesMax; }
    qint64 seriesStartMs() const { return m_seriesStartMs; }
    qint64 seriesEndMs() const { return m_seriesEndMs; }
    QVariantList searchResults() const { return m_searchResults; }
    bool stationsLoading() const { return m_stationsLoading; }
    QVariantList recentStations() const { return m_recentStations; }
    QString apiBase() const { return m_apiBase; }
    QString apiToken() const { return m_apiToken; }
    bool configured() const { return !m_apiBase.isEmpty() && !m_apiToken.isEmpty(); }
    bool needsConfig() const { return m_needsConfig; }

    void setGraphPeriodHours(int hours);
    void setApiBase(const QString &value);
    void setApiToken(const QString &value);

signals:
    void stationChanged();
    void loadingChanged();
    void errorChanged();
    void graphSeriesChanged();
    void searchResultsChanged();
    void stationsLoadingChanged();
    void settingsChanged();
    void connectionTested(bool ok, const QString &message);
    void stationListReady();
    void needsConfigChanged();
    void nearestStationFound();
    void recentStationsChanged();

private:
    void populateFromMetrics(const wasserspiegel::FfiStationMetrics &metrics, bool persist);
    void loadCachedIntoView(const QString &stationId);
    void recomputeSeries();
    void ensureStationListLoaded();
    void setError(const QString &message);
    void setNeedsConfig(bool value);
    void applyDemoData();
    void doFindNearest(double lat, double lon);
    void rememberRecentStation(const QString &id, const QString &name, const QString &water);
    bool createClient();
    QVariantList summariesToVariantList(
        const rust::Vec<wasserspiegel::FfiStationSummary> &list);

    QSettings m_settings;

    std::shared_ptr<ClientHolder> m_client;
    bool m_clientReady = false;
    bool m_hasData = false;

    rust::Vec<wasserspiegel::FfiMeasurementPoint> m_history;

    QString m_stationId;
    QString m_stationName;
    QString m_water;
    double m_km = 0.0;
    double m_currentLevel = 0.0;
    QString m_unit = QStringLiteral("cm");
    double m_change1Day = 0.0;
    double m_change3Day = 0.0;
    double m_change7Day = 0.0;
    bool m_fromCache = false;
    qint64 m_lastUpdatedMs = 0;

    bool m_loading = false;
    QString m_error;

    int m_graphPeriodHours = 72;
    QVariantList m_seriesPoints;
    double m_seriesMin = 0.0;
    double m_seriesMax = 1.0;
    qint64 m_seriesStartMs = 0;
    qint64 m_seriesEndMs = 0;

    QVariantList m_searchResults;
    bool m_stationsLoading = false;
    std::vector<wasserspiegel::FfiStationSummary> m_stationList;

    QVariantList m_recentStations;
    double m_pendingLat = 0.0;
    double m_pendingLon = 0.0;
    bool m_pendingNearest = false;

    QString m_apiBase;
    QString m_apiToken;

    bool m_needsConfig = false;
};

// Installs the Qt message handler that captures qDebug/qWarning output
// (including QML errors) into the in-memory ring buffer exposed via
// AppController::logText().
void wsInstallMessageHandler();

#endif // APPCONTROLLER_H
