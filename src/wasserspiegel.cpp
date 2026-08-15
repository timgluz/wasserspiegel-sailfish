#ifdef QT_QML_DEBUG
#include <QtQuick>
#endif

#include <sailfishapp.h>
#include <QGuiApplication>
#include <QQuickView>
#include <QQmlContext>

#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>

#include "appcontroller.h"

namespace {
void writeStartupMarker()
{
    QStringList paths;
    const QByteArray runtime = qgetenv("XDG_RUNTIME_DIR");
    if (!runtime.isEmpty())
        paths << QString::fromUtf8(runtime) + QStringLiteral("/ws-startup.log");
    paths << QStringLiteral("/tmp/ws-startup.log");
    paths << QStringLiteral("/home/defaultuser/ws-startup.log");
    for (const QString &p : paths) {
        const QByteArray dir = p.toUtf8();
        const int slash = dir.lastIndexOf('/');
        if (slash > 0)
            ::mkdir(dir.left(slash).constData(), 0755);
        const int fd = ::open(p.toUtf8().constData(), O_WRONLY | O_CREAT | O_APPEND, 0644);
        if (fd >= 0) {
            const char *m = "startup\n";
            ::write(fd, m, 8);
            ::close(fd);
        }
    }
}
} // namespace

int main(int argc, char *argv[])
{
    writeStartupMarker();

    QScopedPointer<QGuiApplication> app(SailfishApp::application(argc, argv));
    // install after the app is created so it is not overridden by Sailfish
    wsInstallMessageHandler();

    app->setOrganizationName(QStringLiteral("wasserspiegel"));
    app->setApplicationName(QStringLiteral("wasserspiegel"));

    AppController controller;
    controller.initialize();

    QScopedPointer<QQuickView> view(SailfishApp::createView());
    view->rootContext()->setContextProperty(QStringLiteral("appController"), &controller);
    view->setSource(SailfishApp::pathToMainQml());
    view->show();

    return app->exec();
}
