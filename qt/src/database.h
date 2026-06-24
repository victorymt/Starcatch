#pragma once

#include "models.h"
#include <QSqlDatabase>
#include <QVector>

class Database {
public:
    explicit Database(const QString& path);
    ~Database();

    bool open();
    void migrate();

    // Todo
    QVector<Todo> listTodos(const QString& status = QString());
    QVector<Todo> listTodosByStatuses(const QStringList& statuses);
    void insertTodo(const Todo& todo);
    void updateTodoStatus(const QString& id, TodoStatus status);
    void deleteTodo(const QString& id);

    // Idea
    QVector<Idea> listIdeas(int days);
    void insertIdea(const Idea& idea);
    void deleteIdea(const QString& id);

    // Log
    QVector<LogEntry> listLogs(int days);
    void insertLog(const LogEntry& log);
    void deleteLog(const QString& id);

private:
    QStringList tagsFromJson(const QString& json);
    QString tagsToJson(const QStringList& tags);

    QString m_dbPath;
    QString m_connName;
};
