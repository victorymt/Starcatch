#include "mainwindow.h"
#include "database.h"
#include "todopanel.h"
#include "ideapanel.h"
#include "logpanel.h"
#include "quickinputbar.h"
#include "toastwidget.h"
#include "inputparser.h"
#include "command_plugin.h"
#include "commands/help_command.h"

#include <QVBoxLayout>
#include <QDir>
#include <QUuid>

MainWindow::MainWindow(QWidget* parent)
    : QWidget(parent)
{
    setWindowTitle(QStringLiteral("⭐ Starcatch 星捕"));
    resize(420, 520);
    setMinimumSize(360, 400);

    // Database
    QString dbPath = determineDbPath();
    m_db = new Database(dbPath);
    m_db->open();

    setupUi();
    setupShortcuts();
    registerCommands();

    // Initial load
    m_todoPanel->refresh();
    m_quickInputBar->focusInput();
}

MainWindow::~MainWindow() {
    delete m_db;
}

void MainWindow::setupUi() {
    auto* mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Tab widget
    m_tabWidget = new QTabWidget(this);
    m_todoPanel = new TodoPanel(m_db, this);
    m_ideaPanel = new IdeaPanel(m_db, this);
    m_logPanel  = new LogPanel(m_db, this);

    m_tabWidget->addTab(m_todoPanel, QStringLiteral("📋 Todo"));
    m_tabWidget->addTab(m_ideaPanel, QStringLiteral("💭 Idea"));
    m_tabWidget->addTab(m_logPanel,  QStringLiteral("📓 Log"));

    mainLayout->addWidget(m_tabWidget, 1);

    // Toast (between tabs and input bar)
    m_toast = new ToastWidget(this);
    mainLayout->addWidget(m_toast);

    // Quick input bar
    m_quickInputBar = new QuickInputBar(this);
    mainLayout->addWidget(m_quickInputBar);

    // Connections
    connect(m_tabWidget, &QTabWidget::currentChanged,
            this, &MainWindow::onTabChanged);

    connect(m_quickInputBar, &QuickInputBar::captureRequested,
            this, &MainWindow::quickCapture);

    connect(m_quickInputBar, &QuickInputBar::commandRequested,
            this, &MainWindow::dispatchCommand);
}

void MainWindow::setupShortcuts() {
    auto* escShortcut = new QShortcut(Qt::Key_Escape, this);
    connect(escShortcut, &QShortcut::activated, this, &QWidget::close);
}

void MainWindow::registerCommands() {
    registerCommand<HelpCommand>();
    // Future commands — one line each:
    // registerCommand<SearchCommand>();
    // registerCommand<StatsCommand>();
    // registerCommand<ExportCommand>();
}

QString MainWindow::determineDbPath() {
    const QString home = QDir::homePath();
    const QString dir = home + QStringLiteral("/.local/share/starcatch");
    QDir().mkpath(dir);
    return dir + QStringLiteral("/starcatch.db");
}

void MainWindow::onTabChanged(int index) {
    switch (index) {
        case 0: m_todoPanel->refresh(); break;
        case 1: m_ideaPanel->refresh(); break;
        case 2: m_logPanel->refresh();  break;
    }
    m_quickInputBar->focusInput();
}

void MainWindow::refreshCurrentTab() {
    onTabChanged(m_tabWidget->currentIndex());
}

void MainWindow::quickCapture(const QString& text, QuickKind kind) {
    auto now = QDateTime::currentDateTimeUtc();
    QString id = QUuid::createUuid().toString(QUuid::WithoutBraces);

    switch (kind) {
        case QuickKind::Todo: {
            ParsedInput p = parseTodoInput(text);
            Todo todo;
            todo.id          = id;
            todo.title       = p.title;
            todo.priority    = p.priority;
            todo.status      = TodoStatus::Pending;
            todo.dueDate     = p.dueDate;
            todo.tags        = p.tags;
            todo.createdAt   = now;
            todo.updatedAt   = now;
            m_db->insertTodo(todo);
            break;
        }
        case QuickKind::Idea: {
            Idea idea;
            idea.id        = id;
            idea.title     = text;
            idea.createdAt = now;
            m_db->insertIdea(idea);
            break;
        }
        case QuickKind::Log: {
            LogEntry log;
            log.id        = id;
            log.content   = text;
            log.createdAt = now;
            m_db->insertLog(log);
            break;
        }
    }

    showToast(QStringLiteral("✅ %1").arg(text));
    m_quickInputBar->clearInput();
    m_quickInputBar->focusInput();
    refreshCurrentTab();
}

void MainWindow::showToast(const QString& text) {
    m_toast->showToast(text);
}

void MainWindow::dispatchCommand(const QString& action, const QString& text) {
    CommandPlugin* plugin = CommandRegistry::instance().find(action);
    if (plugin) {
        CommandContext ctx;
        ctx.db = m_db;
        ctx.parentWindow = this;
        ctx.inputBar = m_quickInputBar;
        ctx.showToast = [this](const QString& t) { showToast(t); };
        ctx.refreshCurrentPanel = [this]() { refreshCurrentTab(); };

        bool clearInput = plugin->execute(text, ctx);
        if (clearInput) {
            m_quickInputBar->clearInput();
            m_quickInputBar->focusInput();
        }
    } else {
        showToast(QStringLiteral("❓ 未知命令: /%1").arg(action));
    }
}
