#include <QApplication>
#include "mainwindow.h"
#include "theme.h"

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("starcatch-qt"));
    app.setApplicationVersion(QStringLiteral("0.1.0"));

    if (qEnvironmentVariableIsEmpty("QT_QPA_PLATFORM")) {
        qputenv("QT_QPA_PLATFORM", "wayland");
    }

    ThemeManager::instance().apply(&app);

    MainWindow window;
    window.show();

    return app.exec();
}
