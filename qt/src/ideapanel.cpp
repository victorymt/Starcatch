#include "ideapanel.h"
#include "database.h"

#include <QHBoxLayout>
#include <QLabel>
#include <QToolButton>
#include <QFrame>

// ─── IdeaItemWidget ───

class IdeaItemWidget : public QFrame {
    Q_OBJECT
public:
    IdeaItemWidget(const Idea& idea, QWidget* parent = nullptr)
        : QFrame(parent), m_id(idea.id)
    {
        setProperty("card", true);

        auto* layout = new QHBoxLayout(this);
        layout->setContentsMargins(10, 6, 10, 6);
        layout->setSpacing(8);

        layout->addWidget(new QLabel(QStringLiteral("💡"), this));

        auto* timeLabel = new QLabel(
            idea.createdAt.toLocalTime().toString(QStringLiteral("MM-dd HH:mm")), this);
        timeLabel->setStyleSheet(QStringLiteral("color: #999; font-size: 11px;"));
        timeLabel->setToolTip(idea.createdAt.toUTC().toString(Qt::ISODate));
        layout->addWidget(timeLabel);

        auto* titleLabel = new QLabel(idea.title, this);
        titleLabel->setWordWrap(true);
        layout->addWidget(titleLabel, 1);

        if (!idea.source.isEmpty()) {
            auto* srcLabel = new QLabel(QStringLiteral("(%1)").arg(idea.source), this);
            srcLabel->setStyleSheet(QStringLiteral(
                "color: #999; font-style: italic; font-size: 11px;"));
            layout->addWidget(srcLabel);
        }

        for (const auto& tag : idea.tags) {
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

// ─── IdeaPanel ───

IdeaPanel::IdeaPanel(Database* db, QWidget* parent)
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
    m_daysSlider->setValue(7);
    daysLayout->addWidget(m_daysSlider, 1);

    m_daysLabel = new QLabel(QStringLiteral("7 天"), this);
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

    connect(m_daysSlider, &QSlider::valueChanged, this, &IdeaPanel::onDaysChanged);
}

void IdeaPanel::refresh() {
    auto ideas = m_db->listIdeas(m_days);
    rebuildList(ideas);
}

void IdeaPanel::onDaysChanged(int days) {
    m_days = days;
    m_daysLabel->setText(QStringLiteral("%1 天").arg(days));
    refresh();
}

void IdeaPanel::rebuildList(const QVector<Idea>& ideas) {
    QLayoutItem* item;
    while ((item = m_listLayout->takeAt(0)) != nullptr) {
        if (item->widget()) item->widget()->deleteLater();
        delete item;
    }

    if (ideas.isEmpty()) {
        showEmptyState();
        return;
    }

    for (const auto& idea : ideas) {
        auto* itemWidget = new IdeaItemWidget(idea, m_listWidget);
        connect(itemWidget, &IdeaItemWidget::deleteClicked,
                this, &IdeaPanel::handleDelete);
        m_listLayout->addWidget(itemWidget);
    }

    m_listLayout->addStretch();
}

void IdeaPanel::showEmptyState() {
    auto* emptyWidget = new QWidget(m_listWidget);
    auto* emptyLayout = new QVBoxLayout(emptyWidget);
    emptyLayout->setAlignment(Qt::AlignCenter);

    auto* iconLabel = new QLabel(QStringLiteral("💭 还没有 idea"), emptyWidget);
    iconLabel->setAlignment(Qt::AlignCenter);
    auto* hintLabel = new QLabel(QStringLiteral("切到 Idea 模式，在底部记录吧〜"), emptyWidget);
    hintLabel->setAlignment(Qt::AlignCenter);

    emptyLayout->addStretch();
    emptyLayout->addWidget(iconLabel);
    emptyLayout->addWidget(hintLabel);
    emptyLayout->addStretch();

    m_listLayout->addWidget(emptyWidget);
}

void IdeaPanel::handleDelete(const QString& id) {
    m_db->deleteIdea(id);
    refresh();
}

#include "ideapanel.moc"
