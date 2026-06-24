#include "logpanel.h"
#include "database.h"

#include <QHBoxLayout>
#include <QLabel>
#include <QToolButton>
#include <QFrame>
#include <QMap>

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
        : QFrame(parent), m_id(log.id)
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

        auto* contentLabel = new QLabel(log.content, this);
        contentLabel->setWordWrap(true);
        layout->addWidget(contentLabel, 1);

        // Tags
        for (const auto& tag : log.tags) {
            auto* tagLabel = new QLabel(QStringLiteral("#%1").arg(tag), this);
            tagLabel->setStyleSheet(QStringLiteral(
                "color: #64b5f6; font-size: 10px; background: rgba(100,181,246,0.12);"
                "border-radius: 4px; padding: 1px 5px;"));
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

signals:
    void deleteClicked(const QString& id);

private:
    QString m_id;
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

void LogPanel::handleDelete(const QString& id) {
    m_db->deleteLog(id);
    refresh();
}

#include "logpanel.moc"
