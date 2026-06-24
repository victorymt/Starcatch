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
struct CommandContext;

class MainWindow : public QWidget {
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow();

    // Exposed for CommandContext — plugins call these
    void showToast(const QString& text);
    void refreshCurrentTab();

private slots:
    void onTabChanged(int index);
    void quickCapture(const QString& text, QuickKind kind);

private:
    void setupUi();
    void setupShortcuts();
    void registerCommands();
    void dispatchCommand(const QString& action, const QString& text);
    void updateTabLabels();
    QString determineDbPath();

    Database*      m_db = nullptr;
    QTabWidget*    m_tabWidget = nullptr;
    TodoPanel*     m_todoPanel = nullptr;
    IdeaPanel*     m_ideaPanel = nullptr;
    LogPanel*      m_logPanel = nullptr;
    QuickInputBar* m_quickInputBar = nullptr;
    ToastWidget*   m_toast = nullptr;
};
