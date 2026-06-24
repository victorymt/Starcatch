#pragma once

#include <QWidget>
#include <QLabel>
#include <QTimer>

class ToastWidget : public QWidget {
    Q_OBJECT
public:
    explicit ToastWidget(QWidget* parent = nullptr);

    void showToast(const QString& text);

private:
    QLabel*  m_label;
    QTimer*  m_timer;
};
