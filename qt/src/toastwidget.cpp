#include "toastwidget.h"
#include <QVBoxLayout>

ToastWidget::ToastWidget(QWidget* parent)
    : QWidget(parent)
    , m_label(new QLabel(this))
    , m_timer(new QTimer(this))
{
    setVisible(false);

    m_label->setStyleSheet(QStringLiteral(
        "color: #81c784; font-size: 13px; background: transparent;"
    ));

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(8, 2, 8, 2);
    layout->addWidget(m_label);

    m_timer->setSingleShot(true);
    connect(m_timer, &QTimer::timeout, this, &QWidget::hide);
}

void ToastWidget::showToast(const QString& text) {
    m_label->setText(text);
    show();
    raise();
    m_timer->start(2500);
}
