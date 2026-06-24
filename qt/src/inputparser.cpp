#include "inputparser.h"
#include <QRegularExpression>

QString trimTrailingPunct(const QString& s) {
    if (s.isEmpty()) return s;
    int end = s.size();
    while (end > 0) {
        const QChar c = s.at(end - 1);
        if (c.isPunct()
            || c == QChar(0x3001)   // 、
            || c == QChar(0x3002)   // 。
            || c == QChar(0xFF0C)   // ，
            || c == QChar(0xFF1B)   // ；
            || c == QChar(0xFF1A)   // ：
            || c == QChar(0xFF01)   // ！
            || c == QChar(0xFF1F)   // ？
            || c == QChar(0xFF09)   // ）
            || c == QChar(0xFF3D)   // 】
            || c == QChar(0x300D)   // 》
        ) {
            --end;
        } else {
            break;
        }
    }
    return s.left(end);
}

ParsedInput parseTodoInput(const QString& raw) {
    ParsedInput result;

    const QStringList tokens = raw.split(
        QRegularExpression(QStringLiteral("\\s+")),
        Qt::SkipEmptyParts
    );

    QStringList titleParts;

    for (int i = 0; i < tokens.size(); ++i) {
        const QString& token = tokens[i];

        // Priority keywords
        if (token == QStringLiteral("P0")) {
            result.priority = Priority::P0;
        } else if (token == QStringLiteral("P1")) {
            result.priority = Priority::P1;
        } else if (token == QStringLiteral("P3")) {
            result.priority = Priority::P3;
        } else if (token == QStringLiteral("P2")) {
            result.priority = Priority::P2;
        }
        // due: / due： prefix — value may be in same token or the next
        else if (token.startsWith(QStringLiteral("due:"))
              || token.startsWith(QStringLiteral("due："))
        ) {
            QString val = token.mid(4).trimmed();
            if (val.isEmpty() && i + 1 < tokens.size()) {
                val = tokens[++i];
            }
            if (!val.isEmpty()) {
                result.dueDate = val;
            }
        }
        // #tag — strip leading # then trim trailing punctuation
        else if (token.startsWith(QChar('#'))) {
            QString tag = trimTrailingPunct(token.mid(1).trimmed());
            if (!tag.isEmpty()) {
                result.tags.append(tag);
            }
        }
        // Plain title word
        else {
            titleParts.append(token);
        }
    }

    // Reconstruct title
    if (titleParts.isEmpty()) {
        result.title = raw;
    } else {
        result.title = titleParts.join(QChar(' '));
    }

    return result;
}
