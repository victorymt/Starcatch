#pragma once
#include <QWidget>
#include <QScrollArea>
#include <QVBoxLayout>
#include "models.h"

class Database;

class AllPanel : public QWidget {
    Q_OBJECT
public:
    explicit AllPanel(Database* db, QWidget* parent = nullptr);
    void refresh();
    void showSearchResults(const QString& query);

private:
    void rebuildList(const QString& searchFilter = QString());
    void showEmptyState();
    void handleDeleteTodo(const QString& id);
    void handleDeleteIdea(const QString& id);
    void handleDeleteLog(const QString& id);
    void handleToggleTodo(const QString& id, bool done);

    Database*    m_db;
    QScrollArea* m_scrollArea;
    QWidget*     m_listWidget;
    QVBoxLayout* m_listLayout;
};
