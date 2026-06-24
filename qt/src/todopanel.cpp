#include "todopanel.h"
#include "database.h"

#include <QHBoxLayout>
#include <QCheckBox>
#include <QLabel>
#include <QPushButton>
#include <QFrame>
#include <QToolButton>

// ─── Helpers ───

static QString priorityColor(Priority p) {
    switch (p) {
        case Priority::P0: return QStringLiteral("#e53935");
        case Priority::P1: return QStringLiteral("#fdd835");
        case Priority::P2: return QStringLiteral("#43a047");
        case Priority::P3: return QStringLiteral("#9e9e9e");
    }
    return QStringLiteral("#43a047");
}

// ─── TodoItemWidget ───

class TodoItemWidget : public QFrame {
    Q_OBJECT
public:
    TodoItemWidget(const Todo& todo, QWidget* parent = nullptr)
        : QFrame(parent), m_id(todo.id)
    {
        auto* layout = new QHBoxLayout(this);
        layout->setContentsMargins(4, 2, 4, 2);

        // Priority badge
        auto* badge = new QLabel(priorityToString(todo.priority), this);
        badge->setStyleSheet(QStringLiteral("color: %1; font-weight: bold; font-size: 11px;")
            .arg(priorityColor(todo.priority)));
        badge->setFixedWidth(24);
        layout->addWidget(badge);

        // Checkbox
        auto* cb = new QCheckBox(this);
        bool isDone = todo.status == TodoStatus::Done;
        bool isArchived = todo.status == TodoStatus::Archived;
        cb->setChecked(isDone);
        cb->setEnabled(!isArchived);
        connect(cb, &QCheckBox::toggled, this, [this](bool checked) {
            emit toggled(m_id, checked);
        });
        layout->addWidget(cb);

        // Title
        auto* titleLabel = new QLabel(todo.title, this);
        if (isDone) {
            titleLabel->setText(QStringLiteral("<s>%1</s>").arg(todo.title.toHtmlEscaped()));
            titleLabel->setStyleSheet(QStringLiteral("color: #888;"));
        } else if (isArchived) {
            titleLabel->setStyleSheet(QStringLiteral("color: #555;"));
        }
        titleLabel->setWordWrap(true);
        layout->addWidget(titleLabel, 1);

        // Right-side items
        // Delete button
        auto* delBtn = new QToolButton(this);
        delBtn->setText(QStringLiteral("🗑")); // 🗑 as bytes
        delBtn->setAutoRaise(true);
        connect(delBtn, &QToolButton::clicked, this, [this]() {
            emit deleteClicked(m_id);
        });
        layout->addWidget(delBtn);

        // Archive button (only for non-archived)
        if (!isArchived) {
            auto* archiveBtn = new QToolButton(this);
            archiveBtn->setText(QStringLiteral("📦")); // 📦 as bytes
            archiveBtn->setAutoRaise(true);
            connect(archiveBtn, &QToolButton::clicked, this, [this]() {
                emit archiveClicked(m_id);
            });
            layout->addWidget(archiveBtn);
        }

        // Due date
        if (!todo.dueDate.isEmpty()) {
            auto* dueLabel = new QLabel(todo.dueDate, this);
            dueLabel->setStyleSheet(QStringLiteral("color: #64b5f6; font-size: 11px;"));
            layout->addWidget(dueLabel);
        }

        // Tags
        for (const auto& tag : todo.tags) {
            auto* tagLabel = new QLabel(QStringLiteral("#%1").arg(tag), this);
            tagLabel->setStyleSheet(QStringLiteral("color: #64b5f6; font-size: 11px;"));
            layout->addWidget(tagLabel);
        }

        // Background tint
        if (isArchived) {
            setStyleSheet(QStringLiteral("TodoItemWidget { background: rgba(40,40,40,20); }"));
        } else if (isDone) {
            setStyleSheet(QStringLiteral("TodoItemWidget { background: rgba(30,60,30,20); }"));
        }
    }

signals:
    void toggled(const QString& id, bool checked);
    void deleteClicked(const QString& id);
    void archiveClicked(const QString& id);

private:
    QString m_id;
};

// ─── TodoPanel ───

TodoPanel::TodoPanel(Database* db, QWidget* parent)
    : QWidget(parent), m_db(db)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);

    // Filter chips
    auto* filterWidget = new QWidget(this);
    auto* filterLayout = new QHBoxLayout(filterWidget);
    filterLayout->setContentsMargins(4, 4, 4, 4);

    m_filterGroup = new QButtonGroup(this);
    m_filterGroup->setExclusive(true);

    auto* activeBtn = new QPushButton(QStringLiteral("📋 待办+完成"), filterWidget);
    activeBtn->setCheckable(true);
    activeBtn->setChecked(true);

    auto* pendingBtn = new QPushButton(QStringLiteral("⬜ 仅待办"), filterWidget);
    pendingBtn->setCheckable(true);

    auto* allBtn = new QPushButton(QStringLiteral("📦 全部"), filterWidget);
    allBtn->setCheckable(true);

    m_filterGroup->addButton(activeBtn, 0);
    m_filterGroup->addButton(pendingBtn, 1);
    m_filterGroup->addButton(allBtn, 2);

    filterLayout->addWidget(activeBtn);
    filterLayout->addWidget(pendingBtn);
    filterLayout->addWidget(allBtn);
    filterLayout->addStretch();

    layout->addWidget(filterWidget);

    // Separator
    auto* sep = new QFrame(this);
    sep->setFrameShape(QFrame::HLine);
    sep->setFrameShadow(QFrame::Sunken);
    layout->addWidget(sep);

    // Scroll area
    m_scrollArea = new QScrollArea(this);
    m_scrollArea->setWidgetResizable(true);
    m_scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    m_scrollArea->setFrameShape(QFrame::NoFrame);

    m_listWidget = new QWidget();
    m_listLayout = new QVBoxLayout(m_listWidget);
    m_listLayout->setAlignment(Qt::AlignTop);
    m_listLayout->setContentsMargins(2, 2, 2, 2);
    m_scrollArea->setWidget(m_listWidget);

    layout->addWidget(m_scrollArea, 1);

    connect(m_filterGroup, QOverload<int>::of(&QButtonGroup::idClicked),
            this, [this](int) { onFilterChanged(); });
}

void TodoPanel::refresh() {
    QStringList statuses;

    switch (m_currentFilter) {
        case TodoFilter::Active:
            statuses << QStringLiteral("pending") << QStringLiteral("done");
            break;
        case TodoFilter::Pending:
            statuses << QStringLiteral("pending");
            break;
        case TodoFilter::All:
            statuses << QStringLiteral("pending") << QStringLiteral("done") << QStringLiteral("archived");
            break;
    }

    auto todos = m_db->listTodosByStatuses(statuses);

    std::sort(todos.begin(), todos.end(), [](const Todo& a, const Todo& b) {
        int pa = priorityOrder(a.priority);
        int pb = priorityOrder(b.priority);
        if (pa != pb) return pa < pb;
        return a.createdAt > b.createdAt;
    });

    rebuildList(todos);
}

void TodoPanel::onFilterChanged() {
    int id = m_filterGroup->checkedId();
    switch (id) {
        case 0: m_currentFilter = TodoFilter::Active;  break;
        case 1: m_currentFilter = TodoFilter::Pending; break;
        case 2: m_currentFilter = TodoFilter::All;     break;
        default: m_currentFilter = TodoFilter::Active; break;
    }
    refresh();
}

void TodoPanel::rebuildList(const QVector<Todo>& todos) {
    QLayoutItem* item;
    while ((item = m_listLayout->takeAt(0)) != nullptr) {
        if (item->widget()) item->widget()->deleteLater();
        delete item;
    }

    if (todos.isEmpty()) {
        showEmptyState();
        return;
    }

    for (const auto& todo : todos) {
        auto* itemWidget = new TodoItemWidget(todo, m_listWidget);

        connect(itemWidget, &TodoItemWidget::toggled,
                this, &TodoPanel::handleToggle);
        connect(itemWidget, &TodoItemWidget::deleteClicked,
                this, &TodoPanel::handleDelete);
        connect(itemWidget, &TodoItemWidget::archiveClicked,
                this, &TodoPanel::handleArchive);

        m_listLayout->addWidget(itemWidget);
    }

    m_listLayout->addStretch();
}

void TodoPanel::showEmptyState() {
    auto* emptyWidget = new QWidget(m_listWidget);
    auto* emptyLayout = new QVBoxLayout(emptyWidget);
    emptyLayout->setAlignment(Qt::AlignCenter);

    auto* iconLabel = new QLabel(QStringLiteral("✨ 还没有 todo"), emptyWidget);
    iconLabel->setAlignment(Qt::AlignCenter);
    auto* hintLabel = new QLabel(QStringLiteral("在底部的输入框添加吧〜"), emptyWidget);
    hintLabel->setAlignment(Qt::AlignCenter);

    emptyLayout->addStretch();
    emptyLayout->addWidget(iconLabel);
    emptyLayout->addWidget(hintLabel);
    emptyLayout->addStretch();

    m_listLayout->addWidget(emptyWidget);
}

void TodoPanel::handleToggle(const QString& id, bool done) {
    m_db->updateTodoStatus(id, done ? TodoStatus::Done : TodoStatus::Pending);
    refresh();
}

void TodoPanel::handleDelete(const QString& id) {
    m_db->deleteTodo(id);
    refresh();
}

void TodoPanel::handleArchive(const QString& id) {
    m_db->updateTodoStatus(id, TodoStatus::Archived);
    refresh();
}

#include "todopanel.moc"
