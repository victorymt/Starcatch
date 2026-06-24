#include <QApplication>
#include "mainwindow.h"

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("starcatch-qt"));
    app.setApplicationVersion(QStringLiteral("0.1.0"));

    // Use Wayland by default, fall back to X11
    if (qEnvironmentVariableIsEmpty("QT_QPA_PLATFORM")) {
        qputenv("QT_QPA_PLATFORM", "wayland");
    }

    MainWindow window;
    window.show();

    return app.exec();
}
