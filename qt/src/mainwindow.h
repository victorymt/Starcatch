#pragma once

#include <QWidget>
#include <QTabWidget>
#include <QShortcut>
#include "models.h"

class Database;
class TodoPanel;
class IdeaPanel;
class LogPanel;
class QuickInputBar;
class ToastWidget;

class MainWindow : public QWidget {
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow();

private slots:
    void onTabChanged(int index);
    void refreshCurrentTab();
    void quickCapture(const QString& text, QuickKind kind);
    void handleCommand(const QString& action, const QString& text);

private:
    void setupUi();
    void setupShortcuts();
    void showToast(const QString& text);
    QString determineDbPath();

    Database*      m_db = nullptr;
    QTabWidget*    m_tabWidget = nullptr;
    TodoPanel*     m_todoPanel = nullptr;
    IdeaPanel*     m_ideaPanel = nullptr;
    LogPanel*      m_logPanel = nullptr;
    QuickInputBar* m_quickInputBar = nullptr;
    ToastWidget*   m_toast = nullptr;
};
