#include "database.h"

#include <QSqlQuery>
#include <QSqlError>
#include <QJsonDocument>
#include <QJsonArray>
#include <QUuid>
#include <QDebug>

Database::Database(const QString& path)
    : m_dbPath(path)
    , m_connName(QStringLiteral("starcatch_conn"))
{}

Database::~Database() {
    if (QSqlDatabase::contains(m_connName)) {
        QSqlDatabase::database(m_connName).close();
        QSqlDatabase::removeDatabase(m_connName);
    }
}

bool Database::open() {
    QSqlDatabase db = QSqlDatabase::addDatabase(QStringLiteral("QSQLITE"), m_connName);
    db.setDatabaseName(m_dbPath);
    if (!db.open()) {
        qWarning() << "Failed to open database:" << db.lastError().text();
        return false;
    }

    QSqlQuery q(db);
    q.exec(QStringLiteral("PRAGMA journal_mode = WAL"));
    q.exec(QStringLiteral("PRAGMA foreign_keys = ON"));

    migrate();
    return true;
}

void Database::migrate() {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    q.exec(QStringLiteral(
        "CREATE TABLE IF NOT EXISTS todos ("
        "  id          TEXT PRIMARY KEY,"
        "  title       TEXT NOT NULL,"
        "  description TEXT,"
        "  priority    TEXT NOT NULL DEFAULT 'P2',"
        "  status      TEXT NOT NULL DEFAULT 'pending',"
        "  due_date    TEXT,"
        "  tags        TEXT NOT NULL DEFAULT '[]',"
        "  project     TEXT,"
        "  created_at  TEXT NOT NULL,"
        "  updated_at  TEXT NOT NULL"
        ")"
    ));

    q.exec(QStringLiteral(
        "CREATE TABLE IF NOT EXISTS ideas ("
        "  id              TEXT PRIMARY KEY,"
        "  title           TEXT NOT NULL,"
        "  content         TEXT,"
        "  source          TEXT,"
        "  context_window  TEXT,"
        "  tags            TEXT NOT NULL DEFAULT '[]',"
        "  project         TEXT,"
        "  created_at      TEXT NOT NULL"
        ")"
    ));

    q.exec(QStringLiteral(
        "CREATE TABLE IF NOT EXISTS logs ("
        "  id          TEXT PRIMARY KEY,"
        "  content     TEXT NOT NULL,"
        "  mood        TEXT,"
        "  tags        TEXT NOT NULL DEFAULT '[]',"
        "  project     TEXT,"
        "  created_at  TEXT NOT NULL,"
        "  updated_at  TEXT"
        ")"
    ));

    q.exec(QStringLiteral("CREATE INDEX IF NOT EXISTS idx_todos_status ON todos(status)"));
    q.exec(QStringLiteral("CREATE INDEX IF NOT EXISTS idx_todos_priority ON todos(priority)"));
    q.exec(QStringLiteral("CREATE INDEX IF NOT EXISTS idx_ideas_created ON ideas(created_at)"));
    q.exec(QStringLiteral("CREATE INDEX IF NOT EXISTS idx_logs_created ON logs(created_at)"));

    // Idempotent column additions for older DBs created without `project`.
    addColumnIfMissing(QStringLiteral("ideas"), QStringLiteral("project"), QStringLiteral("TEXT"));
    addColumnIfMissing(QStringLiteral("logs"), QStringLiteral("project"), QStringLiteral("TEXT"));
}

void Database::addColumnIfMissing(const QString& table, const QString& column, const QString& type) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery check(db);
    check.prepare(QStringLiteral("PRAGMA table_info(%1)").arg(table));
    if (!check.exec()) {
        qWarning() << "PRAGMA table_info failed for" << table << ":" << check.lastError().text();
        return;
    }
    bool hasColumn = false;
    while (check.next()) {
        if (check.value(1).toString().compare(column, Qt::CaseInsensitive) == 0) {
            hasColumn = true;
            break;
        }
    }
    if (!hasColumn) {
        QSqlQuery alter(db);
        alter.prepare(QStringLiteral("ALTER TABLE %1 ADD COLUMN %2 %3").arg(table, column, type));
        if (!alter.exec()) {
            qWarning() << "ALTER TABLE" << table << "ADD COLUMN" << column << "failed:" << alter.lastError().text();
        }
    }
}

// Timestamp format used when writing to the DB. Matches Rust's
// `chrono::DateTime::to_rfc3339()` output (`2026-06-29T12:00:00+00:00`) so the
// CLI/TUI and Qt GUI can round-trip rows created by either side.
static const QString kRfc3339 = QStringLiteral("yyyy-MM-ddTHH:mm:ss+00:00");

static QString toDbTimestamp(const QDateTime& dt) {
    return dt.toUTC().toString(kRfc3339);
}

// ─── Tag JSON helpers ───

QStringList Database::tagsFromJson(const QString& json) {
    QJsonDocument doc = QJsonDocument::fromJson(json.toUtf8());
    QStringList tags;
    if (doc.isArray()) {
        for (const auto& val : doc.array()) {
            tags.append(val.toString());
        }
    }
    return tags;
}

QString Database::tagsToJson(const QStringList& tags) {
    QJsonArray arr;
    for (const auto& tag : tags) {
        arr.append(tag);
    }
    return QString::fromUtf8(QJsonDocument(arr).toJson(QJsonDocument::Compact));
}

// ─── Todo ───

QVector<Todo> Database::listTodos(const QString& status) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    if (status.isEmpty()) {
        q.prepare(QStringLiteral(
            "SELECT * FROM todos ORDER BY"
            " CASE priority WHEN 'P0' THEN 0 WHEN 'P1' THEN 1 WHEN 'P2' THEN 2 WHEN 'P3' THEN 3 END,"
            " created_at DESC"
        ));
        q.exec();
    } else {
        q.prepare(QStringLiteral(
            "SELECT * FROM todos WHERE status = ? ORDER BY"
            " CASE priority WHEN 'P0' THEN 0 WHEN 'P1' THEN 1 WHEN 'P2' THEN 2 WHEN 'P3' THEN 3 END,"
            " created_at DESC"
        ));
        q.addBindValue(status);
        q.exec();
    }

    QVector<Todo> todos;
    while (q.next()) {
        Todo t;
        t.id          = q.value(QStringLiteral("id")).toString();
        t.title       = q.value(QStringLiteral("title")).toString();
        t.description = q.value(QStringLiteral("description")).toString();
        t.priority    = stringToPriority(q.value(QStringLiteral("priority")).toString());
        t.status      = stringToStatus(q.value(QStringLiteral("status")).toString());
        t.dueDate     = q.value(QStringLiteral("due_date")).toString();
        t.tags        = tagsFromJson(q.value(QStringLiteral("tags")).toString());
        t.project     = q.value(QStringLiteral("project")).toString();
        t.createdAt   = QDateTime::fromString(q.value(QStringLiteral("created_at")).toString(), Qt::ISODate);
        t.updatedAt   = QDateTime::fromString(q.value(QStringLiteral("updated_at")).toString(), Qt::ISODate);
        todos.append(t);
    }
    return todos;
}

QVector<Todo> Database::listTodosByStatuses(const QStringList& statuses) {
    QVector<Todo> all;
    for (const auto& s : statuses) {
        all.append(listTodos(s));
    }
    return all;
}

void Database::insertTodo(const Todo& todo) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral(
        "INSERT INTO todos (id, title, description, priority, status, due_date, tags, project, created_at, updated_at)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ));
    q.addBindValue(todo.id);
    q.addBindValue(todo.title);
    q.addBindValue(todo.description.isEmpty() ? QVariant() : todo.description);
    q.addBindValue(priorityToString(todo.priority));
    q.addBindValue(statusToString(todo.status));
    q.addBindValue(todo.dueDate.isEmpty() ? QVariant() : todo.dueDate);
    q.addBindValue(tagsToJson(todo.tags));
    q.addBindValue(todo.project.isEmpty() ? QVariant() : todo.project);
    q.addBindValue(toDbTimestamp(todo.createdAt));
    q.addBindValue(toDbTimestamp(todo.updatedAt));
    if (!q.exec()) {
        qWarning() << "insertTodo failed:" << q.lastError().text();
    }
}

void Database::updateTodoStatus(const QString& id, TodoStatus status) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    auto now = toDbTimestamp(QDateTime::currentDateTimeUtc());
    q.prepare(QStringLiteral("UPDATE todos SET status = ?, updated_at = ? WHERE id = ?"));
    q.addBindValue(statusToString(status));
    q.addBindValue(now);
    q.addBindValue(id);
    if (!q.exec()) {
        qWarning() << "updateTodoStatus failed:" << q.lastError().text();
    }
}

void Database::deleteTodo(const QString& id) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral("DELETE FROM todos WHERE id = ?"));
    q.addBindValue(id);
    if (!q.exec()) {
        qWarning() << "deleteTodo failed:" << q.lastError().text();
    }
}

// ─── Idea ───

QVector<Idea> Database::listIdeas(int days) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    q.prepare(QStringLiteral(
        "SELECT * FROM ideas WHERE datetime(created_at) >= datetime('now', ?) ORDER BY created_at DESC"
    ));
    q.addBindValue(QString(QStringLiteral("-%1 days")).arg(days));
    q.exec();

    QVector<Idea> ideas;
    while (q.next()) {
        Idea idea;
        idea.id            = q.value(QStringLiteral("id")).toString();
        idea.title         = q.value(QStringLiteral("title")).toString();
        idea.content       = q.value(QStringLiteral("content")).toString();
        idea.source        = q.value(QStringLiteral("source")).toString();
        idea.contextWindow = q.value(QStringLiteral("context_window")).toString();
        idea.tags          = tagsFromJson(q.value(QStringLiteral("tags")).toString());
        idea.project       = q.value(QStringLiteral("project")).toString();
        idea.createdAt     = QDateTime::fromString(q.value(QStringLiteral("created_at")).toString(), Qt::ISODate);
        ideas.append(idea);
    }
    return ideas;
}

void Database::insertIdea(const Idea& idea) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral(
        "INSERT INTO ideas (id, title, content, source, context_window, tags, project, created_at)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    ));
    q.addBindValue(idea.id);
    q.addBindValue(idea.title);
    q.addBindValue(idea.content.isEmpty() ? QVariant() : idea.content);
    q.addBindValue(idea.source.isEmpty() ? QVariant() : idea.source);
    q.addBindValue(idea.contextWindow.isEmpty() ? QVariant() : idea.contextWindow);
    q.addBindValue(tagsToJson(idea.tags));
    q.addBindValue(idea.project.isEmpty() ? QVariant() : idea.project);
    q.addBindValue(toDbTimestamp(idea.createdAt));
    if (!q.exec()) {
        qWarning() << "insertIdea failed:" << q.lastError().text();
    }
}

void Database::deleteIdea(const QString& id) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral("DELETE FROM ideas WHERE id = ?"));
    q.addBindValue(id);
    if (!q.exec()) {
        qWarning() << "deleteIdea failed:" << q.lastError().text();
    }
}

// ─── Log ───

QVector<LogEntry> Database::listLogs(int days) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);

    q.prepare(QStringLiteral(
        "SELECT * FROM logs WHERE datetime(created_at) >= datetime('now', ?) ORDER BY created_at DESC"
    ));
    q.addBindValue(QString(QStringLiteral("-%1 days")).arg(days));
    q.exec();

    QVector<LogEntry> logs;
    while (q.next()) {
        LogEntry log;
        log.id        = q.value(QStringLiteral("id")).toString();
        log.content   = q.value(QStringLiteral("content")).toString();
        log.mood      = q.value(QStringLiteral("mood")).toString();
        log.tags      = tagsFromJson(q.value(QStringLiteral("tags")).toString());
        log.project   = q.value(QStringLiteral("project")).toString();
        log.createdAt = QDateTime::fromString(q.value(QStringLiteral("created_at")).toString(), Qt::ISODate);
        auto updated  = q.value(QStringLiteral("updated_at")).toString();
        if (!updated.isEmpty()) {
            log.updatedAt = QDateTime::fromString(updated, Qt::ISODate);
        }
        logs.append(log);
    }
    return logs;
}

void Database::insertLog(const LogEntry& log) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral(
        "INSERT INTO logs (id, content, mood, tags, project, created_at, updated_at)"
        " VALUES (?, ?, ?, ?, ?, ?, ?)"
    ));
    q.addBindValue(log.id);
    q.addBindValue(log.content);
    q.addBindValue(log.mood.isEmpty() ? QVariant() : log.mood);
    q.addBindValue(tagsToJson(log.tags));
    q.addBindValue(log.project.isEmpty() ? QVariant() : log.project);
    q.addBindValue(toDbTimestamp(log.createdAt));
    q.addBindValue(log.updatedAt.isValid() ? QVariant(toDbTimestamp(log.updatedAt)) : QVariant());
    if (!q.exec()) {
        qWarning() << "insertLog failed:" << q.lastError().text();
    }
}

void Database::deleteLog(const QString& id) {
    QSqlDatabase db = QSqlDatabase::database(m_connName);
    QSqlQuery q(db);
    q.prepare(QStringLiteral("DELETE FROM logs WHERE id = ?"));
    q.addBindValue(id);
    if (!q.exec()) {
        qWarning() << "deleteLog failed:" << q.lastError().text();
    }
}
