#include "mainwindow.h"
#include "database.h"
#include "todopanel.h"
#include "ideapanel.h"
#include "logpanel.h"
#include "quickinputbar.h"
#include "toastwidget.h"
#include "inputparser.h"

#include <QVBoxLayout>
#include <QDir>
#include <QUuid>
#include <QMessageBox>

MainWindow::MainWindow(QWidget* parent)
    : QWidget(parent)
{
    setWindowTitle(QStringLiteral("⭐ Starcatch 星捕")); // ⭐ Starcatch 星捕
    resize(420, 520);
    setMinimumSize(360, 400);

    // Database
    QString dbPath = determineDbPath();
    m_db = new Database(dbPath);
    m_db->open();

    setupUi();
    setupShortcuts();

    // Initial load
    m_todoPanel->refresh();
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

    m_tabWidget->addTab(m_todoPanel, QStringLiteral("📋 Todo"));  // 📋
    m_tabWidget->addTab(m_ideaPanel, QStringLiteral("💭 Idea"));  // 💭
    m_tabWidget->addTab(m_logPanel,  QStringLiteral("📓 Log"));   // 📓

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
            this, &MainWindow::handleCommand);
}

void MainWindow::setupShortcuts() {
    // Escape closes window
    auto* escShortcut = new QShortcut(Qt::Key_Escape, this);
    connect(escShortcut, &QShortcut::activated, this, &QWidget::close);
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

    showToast(QStringLiteral("✅ %1").arg(text)); // ✅ text
    m_quickInputBar->clearInput();
    m_quickInputBar->focusInput();
    refreshCurrentTab();
}

void MainWindow::showToast(const QString& text) {
    m_toast->showToast(text);
}

void MainWindow::handleCommand(const QString& action, const QString& text) {
    if (action == QStringLiteral("help")) {
        QMessageBox::information(this,
            QStringLiteral("Starcatch 命令"),
            QStringLiteral(
                "可用命令：\n\n"
                "  /t [内容]    切换到 Todo 输入\n"
                "  /i [内容]    切换到 Idea 输入\n"
                "  /l [内容]    切换到 Log 输入\n"
                "  /help        显示此帮助\n\n"
                "快速输入语法 (Todo)：\n"
                "  P0-P3        优先级\n"
                "  due:YYYY-MM-DD  截止日期\n"
                "  #标签        标签\n\n"
                "快捷键：\n"
                "  Enter  提交\n"
                "  Esc    关闭窗口"
            ));
    } else {
        showToast(QStringLiteral("❓ 未知命令: /%1").arg(action));
    }
}
