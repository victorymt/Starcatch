#pragma once

#include <QWidget>
#include <QComboBox>
#include <QLineEdit>
#include <QPushButton>
#include "models.h"

/// Parsed command from input text.
/// /t, /i, /l switch the kind selector. Everything else (e.g. /help, /search)
/// is forwarded as an action command for MainWindow to handle.
struct ParsedCommand {
    bool isCommand = false;          // true if input starts with /
    bool isKindSwitch = false;       // /t /i /l
    QuickKind targetKind = QuickKind::Todo;
    QString action;                  // "help", "search", etc.
    QString text;                    // remaining text after the command word
};

class QuickInputBar : public QWidget {
    Q_OBJECT
public:
    explicit QuickInputBar(QWidget* parent = nullptr);

    void clearInput();
    void focusInput();
    QuickKind currentKind() const;

signals:
    /// Normal capture: user typed regular text and pressed Enter
    void captureRequested(const QString& text, QuickKind kind);
    /// Command capture: user typed /something that isn't /t /i /l
    void commandRequested(const QString& action, const QString& text);

private:
    void updatePlaceholder();
    ParsedCommand parseCommand(const QString& text) const;

    QComboBox*   m_kindCombo;
    QLineEdit*   m_input;
    QPushButton* m_submitBtn;
    bool         m_handlingPrefix = false;
};
