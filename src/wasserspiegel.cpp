#ifdef QT_QML_DEBUG
#include <QtQuick>
#endif

#include <sailfishapp.h>
#include <QGuiApplication>
#include <QQuickView>
#include <QQmlContext>

#include "appcontroller.h"

int main(int argc, char *argv[])
{
    wsInstallMessageHandler();

    QScopedPointer<QGuiApplication> app(SailfishApp::application(argc, argv));
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
