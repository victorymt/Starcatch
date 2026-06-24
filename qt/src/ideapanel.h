#pragma once

#include <QWidget>
#include <QSlider>
#include <QLabel>
#include <QScrollArea>
#include <QVBoxLayout>
#include "models.h"

class Database;

class IdeaPanel : public QWidget {
    Q_OBJECT
public:
    explicit IdeaPanel(Database* db, QWidget* parent = nullptr);

    void refresh();

private:
    void onDaysChanged(int days);
    void rebuildList(const QVector<Idea>& ideas);
    void showEmptyState();
    void handleDelete(const QString& id);

    Database*    m_db;
    QSlider*     m_daysSlider;
    QLabel*      m_daysLabel;
    QScrollArea* m_scrollArea;
    QWidget*     m_listWidget;
    QVBoxLayout* m_listLayout;
    int          m_days = 7;
};
