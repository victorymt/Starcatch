#pragma once

#include <QWidget>
#include <QComboBox>
#include <QLineEdit>
#include <QPushButton>
#include "models.h"

class QuickInputBar : public QWidget {
    Q_OBJECT
public:
    explicit QuickInputBar(QWidget* parent = nullptr);

    void clearInput();
    void focusInput();
    QuickKind currentKind() const;

signals:
    void captureRequested(const QString& text, QuickKind kind);

private:
    void updatePlaceholder();

    QComboBox*   m_kindCombo;
    QLineEdit*   m_input;
    QPushButton* m_submitBtn;
    bool         m_handlingPrefix = false;
};
