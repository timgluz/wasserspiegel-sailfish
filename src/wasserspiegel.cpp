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
    QScopedPointer<QGuiApplication> app(SailfishApp::application(argc, argv));
    // install after the app is created so it is not overridden by Sailfish
    wsInstallMessageHandler();

    app->setOrganizationName(QStringLiteral("org.timgluz"));
    app->setApplicationName(QStringLiteral("harbour-wasserspiegel"));

    AppController controller;
    controller.initialize();

    QScopedPointer<QQuickView> view(SailfishApp::createView());
    view->rootContext()->setContextProperty(QStringLiteral("appController"), &controller);
    view->setSource(SailfishApp::pathToMainQml());
    view->show();

    return app->exec();
}
