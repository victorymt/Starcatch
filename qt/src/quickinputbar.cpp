#include "quickinputbar.h"
#include <QHBoxLayout>

QuickInputBar::QuickInputBar(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(4, 4, 4, 4);

    // Kind selector
    m_kindCombo = new QComboBox(this);
    m_kindCombo->addItem(QStringLiteral("📋 Todo"));
    m_kindCombo->addItem(QStringLiteral("💭 Idea"));
    m_kindCombo->addItem(QStringLiteral("📓 Log"));
    m_kindCombo->setCurrentIndex(0);

    // Text input
    m_input = new QLineEdit(this);
    m_input->setPlaceholderText(QStringLiteral("添加 Todo... (支持 P0-P3, due:, #标签)"));

    // Submit button
    m_submitBtn = new QPushButton(QStringLiteral("➕"), this);
    m_submitBtn->setFixedWidth(36);
    m_submitBtn->setEnabled(false);

    layout->addWidget(m_kindCombo);
    layout->addWidget(m_input, 1);
    layout->addWidget(m_submitBtn);

    // Connections
    connect(m_kindCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &QuickInputBar::updatePlaceholder);

    connect(m_input, &QLineEdit::textChanged, this, [this](const QString& text) {
        // Quick kind switch via /t /i /l prefix
        if (m_handlingPrefix) return;
        QuickKind target = currentKind();
        QString stripped;
        bool matched = false;

        if (text.startsWith(QStringLiteral("/t "))) {
            target = QuickKind::Todo;
            stripped = text.mid(3);
            matched = true;
        } else if (text.startsWith(QStringLiteral("/i "))) {
            target = QuickKind::Idea;
            stripped = text.mid(3);
            matched = true;
        } else if (text.startsWith(QStringLiteral("/l "))) {
            target = QuickKind::Log;
            stripped = text.mid(3);
            matched = true;
        }

        if (matched) {
            m_handlingPrefix = true;
            switch (target) {
                case QuickKind::Todo: m_kindCombo->setCurrentIndex(0); break;
                case QuickKind::Idea: m_kindCombo->setCurrentIndex(1); break;
                case QuickKind::Log:  m_kindCombo->setCurrentIndex(2); break;
            }
            m_input->setText(stripped);
            m_handlingPrefix = false;
        }

        m_submitBtn->setEnabled(!m_input->text().trimmed().isEmpty());
    });

    connect(m_input, &QLineEdit::returnPressed, this, [this]() {
        QString text = m_input->text().trimmed();
        if (!text.isEmpty()) {
            emit captureRequested(text, currentKind());
        }
    });

    connect(m_submitBtn, &QPushButton::clicked, this, [this]() {
        QString text = m_input->text().trimmed();
        if (!text.isEmpty()) {
            emit captureRequested(text, currentKind());
        }
    });
}

void QuickInputBar::clearInput() {
    m_input->clear();
}

void QuickInputBar::focusInput() {
    m_input->setFocus();
}

QuickKind QuickInputBar::currentKind() const {
    switch (m_kindCombo->currentIndex()) {
        case 0:  return QuickKind::Todo;
        case 1:  return QuickKind::Idea;
        default: return QuickKind::Log;
    }
}

void QuickInputBar::updatePlaceholder() {
    switch (currentKind()) {
        case QuickKind::Todo:
            m_input->setPlaceholderText(QStringLiteral("添加 Todo... (支持 P0-P3, due:, #标签)"));
            break;
        case QuickKind::Idea:
            m_input->setPlaceholderText(QStringLiteral("记录 Idea..."));
            break;
        case QuickKind::Log:
            m_input->setPlaceholderText(QStringLiteral("写 Log..."));
            break;
    }
}
