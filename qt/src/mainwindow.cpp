#include "mainwindow.h"
#include "database.h"
#include "todopanel.h"
#include "ideapanel.h"
#include "logpanel.h"
#include "allpanel.h"
#include "quickinputbar.h"
#include "toastwidget.h"
#include "inputparser.h"
#include "command_plugin.h"
#include "commands/help_command.h"
#include "commands/theme_command.h"
#include "commands/search_command.h"
#include "commands/export_command.h"
#include "commands/test_delete_command.h"
#include "commands/stats_command.h"
#include "theme.h"

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

    // Initial load — refresh All and set default tab to Todo
    m_allPanel->refresh();
    m_tabWidget->setCurrentIndex(1); // Start on Todo
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
    m_allPanel  = new AllPanel(m_db, this);

    m_tabWidget->addTab(m_allPanel,  QStringLiteral("🌐 All"));
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

    // Click tag → filter in All tab
    auto filterTag = [this](const QString& tag) {
        m_tabWidget->setCurrentIndex(0);
        m_allPanel->showSearchResults(QStringLiteral("#%1").arg(tag));
    };
    connect(m_todoPanel, &TodoPanel::tagFilterRequested, this, filterTag);
    connect(m_ideaPanel, &IdeaPanel::tagFilterRequested, this, filterTag);
    connect(m_logPanel,  &LogPanel::tagFilterRequested,  this, filterTag);
}

void MainWindow::setupShortcuts() {
    auto* escShortcut = new QShortcut(Qt::Key_Escape, this);
    connect(escShortcut, &QShortcut::activated, this, &QWidget::close);

    auto* themeShortcut = new QShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_T, this);
    connect(themeShortcut, &QShortcut::activated, this, []() {
        ThemeManager::instance().toggle();
    });

    // Ctrl+1/2/3/4 to switch tabs
    for (int i = 0; i < 4; ++i) {
        auto* tabShortcut = new QShortcut(
            QKeySequence(static_cast<Qt::Key>(Qt::Key_1 + i) | Qt::CTRL), this);
        connect(tabShortcut, &QShortcut::activated, this, [this, i]() {
            m_tabWidget->setCurrentIndex(i);
        });
    }

    // Ctrl+N to focus input
    auto* focusShortcut = new QShortcut(
        QKeySequence(Qt::Key_N | Qt::CTRL), this);
    connect(focusShortcut, &QShortcut::activated, this, [this]() {
        m_quickInputBar->focusInput();
    });
}

void MainWindow::registerCommands() {
    registerCommand<HelpCommand>();
    registerCommand<ThemeCommand>();
    registerCommand<SearchCommand>();
    registerCommand<ExportCommand>();
    registerCommand<TestDeleteAllCommand>();
    registerCommand<StatsCommand>();
}

QString MainWindow::determineDbPath() {
    const QString home = QDir::homePath();
    const QString dir = home + QStringLiteral("/.local/share/starcatch");
    QDir().mkpath(dir);
    return dir + QStringLiteral("/starcatch.db");
}

void MainWindow::onTabChanged(int index) {
    switch (index) {
        case 0: m_allPanel->refresh();  break;
        case 1: m_todoPanel->refresh(); break;
        case 2: m_ideaPanel->refresh(); break;
        case 3: m_logPanel->refresh();  break;
    }
    updateTabLabels();
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
            todo.project     = p.project;
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

    // Switch to the matching tab
    int tabIndex = 0;
    switch (kind) {
        case QuickKind::Todo: tabIndex = 1; break;
        case QuickKind::Idea: tabIndex = 2; break;
        case QuickKind::Log:  tabIndex = 3; break;
    }
    m_tabWidget->setCurrentIndex(tabIndex);

    refreshCurrentTab();
    updateTabLabels();
}

void MainWindow::showToast(const QString& text) {
    m_toast->showToast(text);
}

void MainWindow::updateTabLabels() {
    auto todos = m_db->listTodosByStatuses(
        {QStringLiteral("pending"), QStringLiteral("done")});
    int activeCount = 0;
    for (const auto& t : todos) {
        if (t.status == TodoStatus::Pending) ++activeCount;
    }
    int total = activeCount + (int)todos.size() - activeCount; // pending + done

    auto ideas = m_db->listIdeas(7);
    auto logs = m_db->listLogs(1);

    int allCount = (int)todos.size() + ideas.size() + logs.size();
    m_tabWidget->setTabText(0,
        QStringLiteral("🌐 All (%1)").arg(allCount));
    m_tabWidget->setTabText(1,
        QStringLiteral("📋 Todo (%1/%2)").arg(activeCount).arg(todos.size()));
    m_tabWidget->setTabText(2,
        QStringLiteral("💭 Idea (%1)").arg(ideas.size()));
    m_tabWidget->setTabText(3,
        QStringLiteral("📓 Log (%1)").arg(logs.size()));
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
        ctx.searchInAll = [this](const QString& q) {
            m_tabWidget->setCurrentIndex(0);
            m_allPanel->showSearchResults(q);
        };

        bool clearInput = plugin->execute(text, ctx);
        if (clearInput) {
            m_quickInputBar->clearInput();
            m_quickInputBar->focusInput();
        }
    } else {
        showToast(QStringLiteral("❓ 未知命令: /%1").arg(action));
    }
}
