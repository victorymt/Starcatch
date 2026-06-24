#include "inputparser.h"
#include <QRegularExpression>
#include <QDate>
#include <QMap>

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
                // Try natural date first, fall back to raw value
                QString parsed = parseNaturalDate(val);
                result.dueDate = parsed.isEmpty() ? val : parsed;
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

QString parseNaturalDate(const QString& text) {
    QDate today = QDate::currentDate();
    QString t = text.trimmed();

    // Already a date: yyyy-MM-dd
    static QRegularExpression dateRe(QStringLiteral("^\\d{4}-\\d{2}-\\d{2}$"));
    if (dateRe.match(t).hasMatch()) return t;

    // Numeric: N (days later)
    static QRegularExpression numRe(QStringLiteral("^(\\d+)\\s*(天|d|day|days)?(后|後| later)?$"));
    auto m = numRe.match(t);
    if (m.hasMatch()) return today.addDays(m.captured(1).toInt()).toString(QStringLiteral("yyyy-MM-dd"));

    // Day-of-week mapping
    static QMap<QString, int> dowEn, dowZh;
    if (dowEn.isEmpty()) {
        dowEn[QStringLiteral("mon")] = dowEn[QStringLiteral("monday")]    = 1;
        dowEn[QStringLiteral("tue")] = dowEn[QStringLiteral("tuesday")]   = 2;
        dowEn[QStringLiteral("wed")] = dowEn[QStringLiteral("wednesday")] = 3;
        dowEn[QStringLiteral("thu")] = dowEn[QStringLiteral("thursday")]  = 4;
        dowEn[QStringLiteral("fri")] = dowEn[QStringLiteral("friday")]    = 5;
        dowEn[QStringLiteral("sat")] = dowEn[QStringLiteral("saturday")]  = 6;
        dowEn[QStringLiteral("sun")] = dowEn[QStringLiteral("sunday")]    = 7;

        dowZh[QStringLiteral("一")] = 1; dowZh[QStringLiteral("二")] = 2;
        dowZh[QStringLiteral("三")] = 3; dowZh[QStringLiteral("四")] = 4;
        dowZh[QStringLiteral("五")] = 5; dowZh[QStringLiteral("六")] = 6;
        dowZh[QStringLiteral("日")] = 7; dowZh[QStringLiteral("天")] = 7;
    }

    // Absolute keywords
    if (t == QStringLiteral("今天") || t == QStringLiteral("today"))
        return today.toString(QStringLiteral("yyyy-MM-dd"));
    if (t == QStringLiteral("明天") || t == QStringLiteral("tomorrow"))
        return today.addDays(1).toString(QStringLiteral("yyyy-MM-dd"));
    if (t == QStringLiteral("后天") || t == QStringLiteral("後天"))
        return today.addDays(2).toString(QStringLiteral("yyyy-MM-dd"));
    if (t == QStringLiteral("大后天") || t == QStringLiteral("大後天"))
        return today.addDays(3).toString(QStringLiteral("yyyy-MM-dd"));
    if (t == QStringLiteral("昨天") || t == QStringLiteral("yesterday"))
        return today.addDays(-1).toString(QStringLiteral("yyyy-MM-dd"));

    // "next <weekday>" / "下<周X>"
    static QRegularExpression nextEn(QStringLiteral("^next\\s+(\\w+)"), QRegularExpression::CaseInsensitiveOption);
    static QRegularExpression nextZh(QStringLiteral("^下(?:周|星期|礼拜)?(.)"));
    static QRegularExpression thisZh(QStringLiteral("^(?:这|本|这周|本周|这星期|本星期)(?:周|星期|礼拜)?(.)"));

    m = nextEn.match(t);
    if (m.hasMatch() && dowEn.contains(m.captured(1).toLower())) {
        int target = dowEn[m.captured(1).toLower()];
        int delta = target - today.dayOfWeek();
        if (delta <= 0) delta += 7;
        return today.addDays(delta).toString(QStringLiteral("yyyy-MM-dd"));
    }

    m = nextZh.match(t);
    if (m.hasMatch() && dowZh.contains(m.captured(1))) {
        int target = dowZh[m.captured(1)];
        int delta = target - today.dayOfWeek();
        if (delta <= 0) delta += 7;
        return today.addDays(delta).toString(QStringLiteral("yyyy-MM-dd"));
    }

    // "this <weekday>" / "本周X" — this week (may be past)
    m = thisZh.match(t);
    if (m.hasMatch() && dowZh.contains(m.captured(1))) {
        int target = dowZh[m.captured(1)];
        int delta = target - today.dayOfWeek();
        return today.addDays(delta).toString(QStringLiteral("yyyy-MM-dd"));
    }

    // "下周" alone → next Monday
    if (t == QStringLiteral("下周") || t == QStringLiteral("下週") || t == QStringLiteral("next week")) {
        int delta = 8 - today.dayOfWeek(); // next Monday
        return today.addDays(delta).toString(QStringLiteral("yyyy-MM-dd"));
    }

    // "下周<数字>" like 下周一, 下周二 etc — already handled by nextZh above, but also handle 下周一 without 周
    static QRegularExpression nextZh2(QStringLiteral("^下周(.)"));
    m = nextZh2.match(t);
    if (m.hasMatch() && dowZh.contains(m.captured(1))) {
        int target = dowZh[m.captured(1)];
        int delta = target - today.dayOfWeek();
        if (delta <= 0) delta += 7;
        return today.addDays(delta).toString(QStringLiteral("yyyy-MM-dd"));
    }

    return {}; // unrecognized
}
