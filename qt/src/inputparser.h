#pragma once

#include "models.h"

struct ParsedInput {
    QString title;
    Priority priority = Priority::P2;
    QString dueDate;
    QStringList tags;
};

/// Parse a quick-input string into (title, priority, due_date, tags).
/// Recognised tokens:
///   P0 | P1 | P2 | P3  → priority override
///   due:YYYY-MM-DD      → due date (also supports fullwidth colon due：)
///   #tag                → tag (strips trailing ASCII + CJK punctuation)
/// Everything else is joined back into the title.
ParsedInput parseTodoInput(const QString& raw);

/// Strip trailing ASCII and CJK punctuation from a string.
QString trimTrailingPunct(const QString& s);
