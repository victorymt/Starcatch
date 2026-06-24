#pragma once

#include <QWidget>
#include <QButtonGroup>
#include <QScrollArea>
#include <QVBoxLayout>
#include "models.h"

class Database;

class TodoPanel : public QWidget {
    Q_OBJECT
public:
    explicit TodoPanel(Database* db, QWidget* parent = nullptr);

    void refresh();

private:
    void onFilterChanged();
    void rebuildList(const QVector<Todo>& todos);
    void showEmptyState();

    void handleToggle(const QString& id, bool done);
    void handleDelete(const QString& id);
    void handleArchive(const QString& id);
    void handleTitleEdit(const QString& id, const QString& newTitle);

    Database*     m_db;
    QButtonGroup* m_filterGroup;
    QScrollArea*  m_scrollArea;
    QWidget*      m_listWidget;
    QVBoxLayout*  m_listLayout;
    TodoFilter    m_currentFilter = TodoFilter::Active;
};
