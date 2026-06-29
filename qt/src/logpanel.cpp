#include "logpanel.h"
#include "database.h"

#include <QHBoxLayout>
#include <QLabel>
#include <QToolButton>
#include <QFrame>
#include <QLineEdit>
#include <QMouseEvent>
#include <QMap>
#include <QSqlQuery>
#include <QSqlError>
#include <QDateTime>

// ─── Mood icons ───

static QMap<QString, QString> createMoodMap() {
    QMap<QString, QString> m;
    m[QStringLiteral("happy")]   = QStringLiteral("😊"); // 😊
    m[QStringLiteral("sad")]     = QStringLiteral("😢"); // 😢
    m[QStringLiteral("excited")] = QStringLiteral("🤩"); // 🤩
    m[QStringLiteral("angry")]   = QStringLiteral("😤"); // 😤
    m[QStringLiteral("calm")]    = QStringLiteral("😌"); // 😌
    m[QStringLiteral("tired")]   = QStringLiteral("😴"); // 😴
    return m;
}

QString LogPanel::moodIcon(const QString& mood) {
    static const QMap<QString, QString> map = createMoodMap();
    return map.value(mood, mood);
}

// ─── LogItemWidget ───

class LogItemWidget : public QFrame {
    Q_OBJECT
public:
    LogItemWidget(const LogEntry& log, QWidget* parent = nullptr)
        : QFrame(parent), m_id(log.id), m_content(log.content)
    {
        setProperty("card", true);

        auto* layout = new QHBoxLayout(this);
        layout->setContentsMargins(10, 6, 10, 6);
        layout->setSpacing(8);

        layout->addWidget(new QLabel(QStringLiteral("📝"), this));

        auto* timeLabel = new QLabel(
            log.createdAt.toLocalTime().toString(QStringLiteral("MM-dd HH:mm")), this);
        timeLabel->setStyleSheet(QStringLiteral("color: #999; font-size: 11px;"));
        layout->addWidget(timeLabel);

        if (!log.mood.isEmpty()) {
            layout->addWidget(new QLabel(LogPanel::moodIcon(log.mood), this));
        }

        m_contentLabel = new QLabel(log.content, this);
        m_contentLabel->setCursor(Qt::IBeamCursor);
        m_contentLabel->setToolTip(QStringLiteral("双击编辑"));
        m_contentLabel->setWordWrap(true);
        layout->addWidget(m_contentLabel, 1);

        // Tags
        for (const auto& tag : log.tags) {
            auto* tagLabel = new QLabel(
                QStringLiteral("<a href='tag:%1' style='color:#64b5f6;text-decoration:none;'>#%1</a>").arg(tag), this);
            tagLabel->setTextInteractionFlags(Qt::LinksAccessibleByMouse);
            tagLabel->setStyleSheet(QStringLiteral(
                "font-size: 10px; background: rgba(100,181,246,0.12);"
                "border-radius: 4px; padding: 1px 5px;"));
            connect(tagLabel, &QLabel::linkActivated, this, [this](const QString& link) {
                if (link.startsWith(QStringLiteral("tag:")))
                    emit tagClicked(link.mid(4));
            });
            layout->addWidget(tagLabel);
        }

        auto* delBtn = new QToolButton(this);
        delBtn->setText(QStringLiteral("🗑"));
        delBtn->setAutoRaise(true);
        delBtn->setToolTip(QStringLiteral("删除"));
        delBtn->setStyleSheet(QStringLiteral(
            "QToolButton { color: #888; }"
            "QToolButton:hover { color: #e53935; background: rgba(229,57,53,0.15); }"));
        connect(delBtn, &QToolButton::clicked, this, [this]() {
            emit deleteClicked(m_id);
        });
        layout->addWidget(delBtn);
    }

    void startEdit() {
        if (m_editing) return;
        m_editing = true;
        auto* edit = new QLineEdit(m_content, this);
        edit->selectAll();
        edit->setStyleSheet(QStringLiteral("QLineEdit { border-radius: 4px; padding: 2px 6px; }"));
        m_contentLabel->hide();
        auto* lay = qobject_cast<QHBoxLayout*>(layout());
        int idx = lay->indexOf(m_contentLabel);
        lay->insertWidget(idx, edit);
        edit->setFocus();
        auto finish = [this, edit, lay](bool save) {
            if (save) {
                QString t = edit->text().trimmed();
                if (!t.isEmpty() && t != m_content) emit contentEdited(m_id, t);
            }
            lay->removeWidget(edit);
            edit->deleteLater();
            m_contentLabel->show();
            m_editing = false;
        };
        connect(edit, &QLineEdit::returnPressed, this, [finish]() { finish(true); });
        connect(edit, &QLineEdit::editingFinished, this, [finish]() { finish(true); });
    }

protected:
    void mouseDoubleClickEvent(QMouseEvent* ev) override {
        if (m_contentLabel->geometry().contains(ev->pos())) { startEdit(); return; }
        QFrame::mouseDoubleClickEvent(ev);
    }

signals:
    void deleteClicked(const QString& id);
    void tagClicked(const QString& tag);
    void contentEdited(const QString& id, const QString& newContent);

private:
    QString m_id;
    QString m_content;
    QLabel* m_contentLabel = nullptr;
    bool m_editing = false;
};

// ─── LogPanel ───

LogPanel::LogPanel(Database* db, QWidget* parent)
    : QWidget(parent), m_db(db)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);

    auto* daysWidget = new QWidget(this);
    auto* daysLayout = new QHBoxLayout(daysWidget);
    daysLayout->setContentsMargins(4, 4, 4, 4);

    daysLayout->addWidget(new QLabel(QStringLiteral("最近"), this));

    m_daysSlider = new QSlider(Qt::Horizontal, this);
    m_daysSlider->setRange(1, 365);
    m_daysSlider->setValue(1);
    daysLayout->addWidget(m_daysSlider, 1);

    m_daysLabel = new QLabel(QStringLiteral("1 天"), this);
    m_daysLabel->setFixedWidth(50);
    daysLayout->addWidget(m_daysLabel);

    layout->addWidget(daysWidget);

    auto* sep = new QFrame(this);
    sep->setFrameShape(QFrame::HLine);
    sep->setFrameShadow(QFrame::Sunken);
    layout->addWidget(sep);

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

    connect(m_daysSlider, &QSlider::valueChanged, this, &LogPanel::onDaysChanged);
}

void LogPanel::refresh() {
    auto logs = m_db->listLogs(m_days);
    rebuildList(logs);
}

void LogPanel::onDaysChanged(int days) {
    m_days = days;
    m_daysLabel->setText(QStringLiteral("%1 天").arg(days));
    refresh();
}

void LogPanel::rebuildList(const QVector<LogEntry>& logs) {
    QLayoutItem* item;
    while ((item = m_listLayout->takeAt(0)) != nullptr) {
        if (item->widget()) item->widget()->deleteLater();
        delete item;
    }

    if (logs.isEmpty()) {
        showEmptyState();
        return;
    }

    for (const auto& log : logs) {
        auto* itemWidget = new LogItemWidget(log, m_listWidget);
        connect(itemWidget, &LogItemWidget::deleteClicked,
                this, &LogPanel::handleDelete);
        connect(itemWidget, &LogItemWidget::tagClicked,
                this, &LogPanel::tagFilterRequested);
        connect(itemWidget, &LogItemWidget::contentEdited,
                this, &LogPanel::handleContentEdit);
        m_listLayout->addWidget(itemWidget);
    }

    m_listLayout->addStretch();
}

void LogPanel::showEmptyState() {
    auto* emptyWidget = new QWidget(m_listWidget);
    auto* emptyLayout = new QVBoxLayout(emptyWidget);
    emptyLayout->setAlignment(Qt::AlignCenter);

    auto* iconLabel = new QLabel(QStringLiteral("📓 还没有日志"), emptyWidget);
    iconLabel->setAlignment(Qt::AlignCenter);
    auto* hintLabel = new QLabel(QStringLiteral("切到 Log 模式，记录今天的事吧〜"), emptyWidget);
    hintLabel->setAlignment(Qt::AlignCenter);

    emptyLayout->addStretch();
    emptyLayout->addWidget(iconLabel);
    emptyLayout->addWidget(hintLabel);
    emptyLayout->addStretch();

    m_listLayout->addWidget(emptyWidget);
}

void LogPanel::handleContentEdit(const QString& id, const QString& newContent) {
    QSqlDatabase db = QSqlDatabase::database(QStringLiteral("starcatch_conn"));
    QSqlQuery q(db);
    q.prepare(QStringLiteral("UPDATE logs SET content = ?, updated_at = ? WHERE id = ?"));
    q.addBindValue(newContent);
    q.addBindValue(QDateTime::currentDateTimeUtc().toUTC().toString(QStringLiteral("yyyy-MM-ddTHH:mm:ss+00:00")));
    q.addBindValue(id);
    if (!q.exec()) qWarning() << "handleContentEdit(log) failed:" << q.lastError().text();
    refresh();
}

void LogPanel::handleDelete(const QString& id) {
    m_db->deleteLog(id);
    refresh();
}

#include "logpanel.moc"
