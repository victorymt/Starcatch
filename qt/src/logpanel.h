#pragma once

#include <QWidget>
#include <QSlider>
#include <QLabel>
#include <QScrollArea>
#include <QVBoxLayout>
#include "models.h"

class Database;

class LogPanel : public QWidget {
    Q_OBJECT
public:
    explicit LogPanel(Database* db, QWidget* parent = nullptr);

    void refresh();

signals:
    void tagFilterRequested(const QString& tag);

public:
    static QString moodIcon(const QString& mood);

private:
    void onDaysChanged(int days);
    void rebuildList(const QVector<LogEntry>& logs);
    void showEmptyState();
    void handleDelete(const QString& id);

    Database*    m_db;
    QSlider*     m_daysSlider;
    QLabel*      m_daysLabel;
    QScrollArea* m_scrollArea;
    QWidget*     m_listWidget;
    QVBoxLayout* m_listLayout;
    int          m_days = 1;
};
