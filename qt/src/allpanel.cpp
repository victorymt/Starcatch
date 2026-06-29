#include "allpanel.h"
#include "database.h"

#include "logpanel.h"
#include <QHBoxLayout>
#include <QLabel>
#include <QToolButton>
#include <QCheckBox>
#include <QFrame>
#include <algorithm>

// ─── MixedEntry — one row in the All feed ───

struct MixedEntry {
    enum Kind { Todo, Idea, Log };
    Kind kind;
    QString id;
    QString icon;
    QString text;
    QString sub;       // tags / source / mood
    QDateTime createdAt;
    bool isDone = false;
    bool isArchived = false;
};

// ─── AllItemWidget ───

class AllItemWidget : public QFrame {
    Q_OBJECT
public:
    AllItemWidget(const MixedEntry& e, QWidget* parent = nullptr)
        : QFrame(parent), m_entry(e)
    {
        setProperty("card", true);
        auto* layout = new QHBoxLayout(this);
        layout->setContentsMargins(10, 5, 10, 5);
        layout->setSpacing(8);

        // Type icon
        layout->addWidget(new QLabel(e.icon, this));

        // Timestamp
        auto* timeLabel = new QLabel(
            e.createdAt.toLocalTime().toString(QStringLiteral("MM-dd HH:mm")), this);
        timeLabel->setStyleSheet(QStringLiteral("color: #999; font-size: 11px;"));
        layout->addWidget(timeLabel);

        // Checkbox for todos
        if (e.kind == MixedEntry::Todo && !e.isArchived) {
            auto* cb = new QCheckBox(this);
            cb->setChecked(e.isDone);
            connect(cb, &QCheckBox::toggled, this, [this](bool checked) {
                emit todoToggled(m_entry.id, checked);
            });
            layout->addWidget(cb);
        }

        // Text
        auto* textLabel = new QLabel(e.text, this);
        if (e.isDone) {
            textLabel->setText(QStringLiteral("<s>%1</s>").arg(e.text.toHtmlEscaped()));
            textLabel->setStyleSheet(QStringLiteral("color: #777;"));
        }
        textLabel->setWordWrap(true);
        layout->addWidget(textLabel, 1);

        // Sub info (tags/source)
        if (!e.sub.isEmpty()) {
            auto* subLabel = new QLabel(e.sub, this);
            subLabel->setStyleSheet(QStringLiteral("color: #64b5f6; font-size: 10px;"));
            layout->addWidget(subLabel);
        }

        // Delete
        auto* delBtn = new QToolButton(this);
        delBtn->setText(QStringLiteral("🗑"));
        delBtn->setAutoRaise(true);
        delBtn->setStyleSheet(QStringLiteral(
            "QToolButton { color: #888; }"
            "QToolButton:hover { color: #e53935; background: rgba(229,57,53,0.15); }"));
        connect(delBtn, &QToolButton::clicked, this, [this]() {
            emit deleteClicked(m_entry);
        });
        layout->addWidget(delBtn);
    }

signals:
    void deleteClicked(const MixedEntry& e);
    void todoToggled(const QString& id, bool checked);

private:
    MixedEntry m_entry;
};

// ─── AllPanel ───

AllPanel::AllPanel(Database* db, QWidget* parent)
    : QWidget(parent), m_db(db)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);

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
}

void AllPanel::refresh() {
    rebuildList();
}

void AllPanel::showSearchResults(const QString& query) {
    rebuildList(query);
}

static bool matches(const QString& text, const QString& query) {
    return text.toLower().contains(query.toLower());
}

void AllPanel::rebuildList(const QString& searchFilter) {
    QLayoutItem* item;
    while ((item = m_listLayout->takeAt(0)) != nullptr) {
        if (item->widget()) item->widget()->deleteLater();
        delete item;
    }

    QVector<MixedEntry> entries;

    auto tagsToSub = [](const QStringList& tags) -> QString {
        if (tags.isEmpty()) return {};
        QStringList prefixed;
        for (const auto& t : tags) prefixed << QStringLiteral("#%1").arg(t);
        return prefixed.join(QStringLiteral(" "));
    };

    // Todos (active: pending + done)
    auto todos = m_db->listTodosByStatuses(
        {QStringLiteral("pending"), QStringLiteral("done"), QStringLiteral("archived")});
    for (const auto& t : todos) {
        MixedEntry e;
        e.kind = MixedEntry::Todo;
        e.id = t.id;
        e.icon = priorityIcon(t.priority);
        e.text = t.title;
        e.sub = tagsToSub(t.tags);
        e.createdAt = t.createdAt;
        e.isDone = t.status == TodoStatus::Done;
        e.isArchived = t.status == TodoStatus::Archived;
        entries.append(e);
    }

    // Ideas
    auto ideas = m_db->listIdeas(7);
    for (const auto& i : ideas) {
        MixedEntry e;
        e.kind = MixedEntry::Idea;
        e.id = i.id;
        e.icon = QStringLiteral("💡");
        e.text = i.title;
        QStringList parts;
        if (!i.source.isEmpty()) parts << i.source;
        parts << tagsToSub(i.tags);
        e.sub = parts.join(QStringLiteral(" "));
        e.createdAt = i.createdAt;
        entries.append(e);
    }

    // Logs
    auto logs = m_db->listLogs(1);
    for (const auto& l : logs) {
        MixedEntry e;
        e.kind = MixedEntry::Log;
        e.id = l.id;
        e.icon = LogPanel::moodIcon(l.mood).isEmpty() ? QStringLiteral("📝") : LogPanel::moodIcon(l.mood);
        e.text = l.content;
        e.sub = tagsToSub(l.tags);
        e.createdAt = l.createdAt;
        entries.append(e);
    }

    // Filter by search query if active
    if (!searchFilter.isEmpty()) {
        QVector<MixedEntry> filtered;
        for (const auto& e : entries) {
            if (matches(e.text, searchFilter) || matches(e.sub, searchFilter))
                filtered.append(e);
        }
        entries = filtered;
    }

    // Sort by created_at descending
    std::sort(entries.begin(), entries.end(),
              [](const MixedEntry& a, const MixedEntry& b) {
                  return a.createdAt > b.createdAt;
              });

    if (entries.isEmpty()) {
        if (!searchFilter.isEmpty()) {
            auto* emptyWidget = new QWidget(m_listWidget);
            auto* emptyLayout = new QVBoxLayout(emptyWidget);
            emptyLayout->setAlignment(Qt::AlignCenter);
            auto* l = new QLabel(QStringLiteral("🔍 未找到: %1").arg(searchFilter), emptyWidget);
            l->setAlignment(Qt::AlignCenter);
            emptyLayout->addStretch();
            emptyLayout->addWidget(l);
            emptyLayout->addStretch();
            m_listLayout->addWidget(emptyWidget);
        } else {
            showEmptyState();
        }
        return;
    }

    // Group by kind and add section headers
    auto addSection = [&](const QString& title, MixedEntry::Kind kind) {
        QVector<MixedEntry> group;
        for (const auto& e : entries) {
            if (e.kind == kind) group.append(e);
        }
        if (group.isEmpty()) return;

        // Sort group by created_at desc
        std::sort(group.begin(), group.end(),
                  [](const MixedEntry& a, const MixedEntry& b) {
                      return a.createdAt > b.createdAt;
                  });

        // Section header
        auto* header = new QLabel(title, m_listWidget);
        header->setAlignment(Qt::AlignCenter);
        header->setStyleSheet(QStringLiteral(
            "color: #7a7a9a; font-weight: bold; font-size: 15px;"
            "padding: 8px 8px 4px 8px; background: transparent;"
            "border-bottom: 1px solid #2a2a4a; margin: 4px 8px 0px 8px;"));
        m_listLayout->addWidget(header);

        for (const auto& e : group) {
            auto* w = new AllItemWidget(e, m_listWidget);
            connect(w, &AllItemWidget::deleteClicked, this, [this](const MixedEntry& me) {
                switch (me.kind) {
                    case MixedEntry::Todo: handleDeleteTodo(me.id); break;
                    case MixedEntry::Idea: handleDeleteIdea(me.id); break;
                    case MixedEntry::Log:  handleDeleteLog(me.id);  break;
                }
            });
            connect(w, &AllItemWidget::todoToggled, this, &AllPanel::handleToggleTodo);
            m_listLayout->addWidget(w);
        }
    };

    addSection(QStringLiteral("📋 Todo"), MixedEntry::Todo);
    addSection(QStringLiteral("💭 Idea"), MixedEntry::Idea);
    addSection(QStringLiteral("📓 Log"), MixedEntry::Log);

    m_listLayout->addStretch();
}

void AllPanel::showEmptyState() {
    auto* emptyWidget = new QWidget(m_listWidget);
    auto* emptyLayout = new QVBoxLayout(emptyWidget);
    emptyLayout->setAlignment(Qt::AlignCenter);
    auto* l = new QLabel(QStringLiteral("✨ 空空如也"), emptyWidget);
    l->setAlignment(Qt::AlignCenter);
    emptyLayout->addStretch();
    emptyLayout->addWidget(l);
    emptyLayout->addStretch();
    m_listLayout->addWidget(emptyWidget);
}

void AllPanel::handleDeleteTodo(const QString& id) { m_db->deleteTodo(id); refresh(); }
void AllPanel::handleDeleteIdea(const QString& id) { m_db->deleteIdea(id); refresh(); }
void AllPanel::handleDeleteLog(const QString& id)  { m_db->deleteLog(id);  refresh(); }

void AllPanel::handleToggleTodo(const QString& id, bool done) {
    m_db->updateTodoStatus(id, done ? TodoStatus::Done : TodoStatus::Pending);
    refresh();
}

#include "allpanel.moc"
