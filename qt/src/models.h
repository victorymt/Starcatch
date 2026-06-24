#pragma once

#include <QString>
#include <QStringList>
#include <QDateTime>

// ─── Enums (mirrors Rust Priority / TodoStatus) ───

enum class Priority { P0, P1, P2, P3 };
enum class TodoStatus { Pending, Done, Archived };

inline QString priorityToString(Priority p) {
    switch (p) {
        case Priority::P0: return QStringLiteral("P0");
        case Priority::P1: return QStringLiteral("P1");
        case Priority::P2: return QStringLiteral("P2");
        case Priority::P3: return QStringLiteral("P3");
    }
    return QStringLiteral("P2");
}

inline Priority stringToPriority(const QString& s) {
    if (s == QStringLiteral("P0")) return Priority::P0;
    if (s == QStringLiteral("P1")) return Priority::P1;
    if (s == QStringLiteral("P3")) return Priority::P3;
    return Priority::P2;
}

inline QString priorityIcon(Priority p) {
    switch (p) {
        case Priority::P0: return QStringLiteral("🔴");
        case Priority::P1: return QStringLiteral("🟡");
        case Priority::P2: return QStringLiteral("🟢");
        case Priority::P3: return QStringLiteral("⚪");
    }
    return QStringLiteral("🟢");
}

inline int priorityOrder(Priority p) {
    switch (p) {
        case Priority::P0: return 0;
        case Priority::P1: return 1;
        case Priority::P2: return 2;
        case Priority::P3: return 3;
    }
    return 2;
}

inline QString statusToString(TodoStatus s) {
    switch (s) {
        case TodoStatus::Pending:  return QStringLiteral("pending");
        case TodoStatus::Done:     return QStringLiteral("done");
        case TodoStatus::Archived: return QStringLiteral("archived");
    }
    return QStringLiteral("pending");
}

inline TodoStatus stringToStatus(const QString& s) {
    if (s == QStringLiteral("done"))     return TodoStatus::Done;
    if (s == QStringLiteral("archived")) return TodoStatus::Archived;
    return TodoStatus::Pending;
}

// ─── Data structs (mirrors Rust Todo / Idea / Log) ───

struct Todo {
    QString id;
    QString title;
    QString description;
    Priority priority = Priority::P2;
    TodoStatus status = TodoStatus::Pending;
    QString dueDate;
    QStringList tags;
    QString project;
    QDateTime createdAt;
    QDateTime updatedAt;
};

struct Idea {
    QString id;
    QString title;
    QString content;
    QString source;
    QString contextWindow;
    QStringList tags;
    QDateTime createdAt;
};

struct LogEntry {
    QString id;
    QString content;
    QString mood;
    QStringList tags;
    QDateTime createdAt;
    QDateTime updatedAt;
};

// ─── Tab / filter enums ───

enum class PanelTab { Todo, Idea, Log };
enum class TodoFilter { Active, Pending, All };
enum class QuickKind { Todo, Idea, Log };
