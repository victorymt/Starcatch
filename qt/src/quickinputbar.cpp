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
    m_input->setPlaceholderText(QStringLiteral("添加 Todo... (/t /i /l /help)"));
    m_input->setToolTip(QStringLiteral(
        "Commands: /t Todo  /i Idea  /l Log  — type /help for more"
    ));

    // Submit button
    m_submitBtn = new QPushButton(QStringLiteral("➕"), this);
    m_submitBtn->setFixedWidth(36);
    m_submitBtn->setEnabled(false);

    layout->addWidget(m_kindCombo);
    layout->addWidget(m_input, 1);
    layout->addWidget(m_submitBtn);

    // ── Connections ──

    connect(m_kindCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &QuickInputBar::updatePlaceholder);

    // Text change: detect commands, switch kind, enable/disable submit
    connect(m_input, &QLineEdit::textChanged, this, [this](const QString& text) {
        if (m_handlingPrefix) return;

        ParsedCommand cmd = parseCommand(text);
        // Only instant-switch when there's a space after the prefix
        // (/t something). Bare /t /i /l are handled on Enter to avoid
        // conflicting with longer commands like /theme.
        bool hasSpace = text.contains(QChar(' '));
        if (cmd.isCommand && cmd.isKindSwitch && hasSpace) {
            m_handlingPrefix = true;
            switch (cmd.targetKind) {
                case QuickKind::Todo: m_kindCombo->setCurrentIndex(0); break;
                case QuickKind::Idea: m_kindCombo->setCurrentIndex(1); break;
                case QuickKind::Log:  m_kindCombo->setCurrentIndex(2); break;
            }
            m_input->setText(cmd.text);
            m_handlingPrefix = false;
        }

        m_submitBtn->setEnabled(!m_input->text().trimmed().isEmpty());
    });

    // Enter → submit
    connect(m_input, &QLineEdit::returnPressed, this, [this]() {
        QString text = m_input->text().trimmed();
        if (text.isEmpty()) return;

        // Re-parse on submit to handle edge cases (e.g. bare "/t" with no text)
        ParsedCommand cmd = parseCommand(m_input->text());

        if (cmd.isCommand && !cmd.isKindSwitch) {
            // Action command → forward to MainWindow
            emit commandRequested(cmd.action, cmd.text.trimmed());
            clearInput();
            focusInput();
        } else if (cmd.isCommand && cmd.isKindSwitch && cmd.text.trimmed().isEmpty()) {
            // Bare /t /i /l → just switch kind (already handled), don't capture
            clearInput();
            focusInput();
        } else {
            // Normal capture
            emit captureRequested(text, currentKind());
        }
    });

    // ➕ button → same as Enter
    connect(m_submitBtn, &QPushButton::clicked, this, [this]() {
        QString text = m_input->text().trimmed();
        if (text.isEmpty()) return;

        ParsedCommand cmd = parseCommand(m_input->text());

        if (cmd.isCommand && !cmd.isKindSwitch) {
            emit commandRequested(cmd.action, cmd.text.trimmed());
            clearInput();
            focusInput();
        } else if (cmd.isCommand && cmd.isKindSwitch && cmd.text.trimmed().isEmpty()) {
            clearInput();
            focusInput();
        } else {
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
            m_input->setPlaceholderText(QStringLiteral("添加 Todo... (/t /i /l /help)"));
            break;
        case QuickKind::Idea:
            m_input->setPlaceholderText(QStringLiteral("记录 Idea... (/t /i /l /help)"));
            break;
        case QuickKind::Log:
            m_input->setPlaceholderText(QStringLiteral("写 Log... (/t /i /l /help)"));
            break;
    }
}

// ─── Command parser ───

ParsedCommand QuickInputBar::parseCommand(const QString& text) const {
    ParsedCommand cmd;

    if (!text.startsWith(QChar('/'))) {
        return cmd; // isCommand = false
    }

    // Split into command word + rest
    int spaceIdx = text.indexOf(QChar(' '));
    QString cmdWord = (spaceIdx > 0) ? text.left(spaceIdx) : text;
    QString rest = (spaceIdx > 0) ? text.mid(spaceIdx + 1) : QString();

    // Check for kind-switch commands
    if (cmdWord == QStringLiteral("/t")) {
        cmd.isCommand = true;
        cmd.isKindSwitch = true;
        cmd.targetKind = QuickKind::Todo;
        cmd.text = rest;
    } else if (cmdWord == QStringLiteral("/i")) {
        cmd.isCommand = true;
        cmd.isKindSwitch = true;
        cmd.targetKind = QuickKind::Idea;
        cmd.text = rest;
    } else if (cmdWord == QStringLiteral("/l")) {
        cmd.isCommand = true;
        cmd.isKindSwitch = true;
        cmd.targetKind = QuickKind::Log;
        cmd.text = rest;
    } else if (cmdWord.length() > 1) {
        // Unknown /command → forward as action
        cmd.isCommand = true;
        cmd.isKindSwitch = false;
        cmd.action = cmdWord.mid(1); // strip leading /
        cmd.text = rest;
    }

    return cmd;
}
