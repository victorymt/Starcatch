#pragma once
#include "../command_plugin.h"
#include <QDir>
#include <QFile>
#include <QTextStream>
#include <QMessageBox>
#include <QDateTime>

class ExportCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("export"); }
    QString description() const override { return QStringLiteral("导出数据为 Markdown"); }

    bool execute(const QString&, CommandContext& ctx) override {
        const QString dir = QDir::homePath() + QStringLiteral("/.local/share/starcatch");
        QDir().mkpath(dir);

        QString timestamp = QDateTime::currentDateTime().toString(QStringLiteral("yyyyMMdd-HHmmss"));
        QString path = dir + QStringLiteral("/export-") + timestamp + QStringLiteral(".md");

        QFile f(path);
        if (!f.open(QIODevice::WriteOnly | QIODevice::Text)) {
            ctx.showToast(QStringLiteral("❌ 导出失败"));
            return true;
        }

        QTextStream out(&f);
        out << QStringLiteral("# Starcatch 导出\n\n");
        out << QStringLiteral("导出时间: %1\n\n").arg(
            QDateTime::currentDateTime().toString(Qt::ISODate));

        // Todos
        auto todos = ctx.db->listTodosByStatuses(
            {QStringLiteral("pending"), QStringLiteral("done"), QStringLiteral("archived")});
        out << QStringLiteral("## 📋 Todo (%1)\n\n").arg(todos.size());
        for (const auto& t : todos) {
            QString status = (t.status == TodoStatus::Done) ? QStringLiteral("[x]") :
                             (t.status == TodoStatus::Archived) ? QStringLiteral("[~]") :
                             QStringLiteral("[ ]");
            out << QStringLiteral("- %1 **%2** %3").arg(status, t.title, priorityToString(t.priority));
            if (!t.dueDate.isEmpty()) out << QStringLiteral(" | due: %1").arg(t.dueDate);
            if (!t.tags.isEmpty()) out << QStringLiteral(" | %1").arg(t.tags.join(QStringLiteral(", ")));
            out << QStringLiteral("\n");
        }

        // Ideas
        auto ideas = ctx.db->listIdeas(365);
        out << QStringLiteral("\n## 💭 Idea (%1)\n\n").arg(ideas.size());
        for (const auto& i : ideas) {
            out << QStringLiteral("- %1").arg(i.title);
            if (!i.source.isEmpty()) out << QStringLiteral(" (来源: %1)").arg(i.source);
            if (!i.tags.isEmpty()) out << QStringLiteral(" | %1").arg(i.tags.join(QStringLiteral(", ")));
            out << QStringLiteral("\n");
        }

        // Logs
        auto logs = ctx.db->listLogs(365);
        out << QStringLiteral("\n## 📓 Log (%1)\n\n").arg(logs.size());
        for (const auto& l : logs) {
            out << QStringLiteral("- %1 %2\n").arg(
                l.createdAt.toLocalTime().toString(QStringLiteral("MM-dd HH:mm")),
                l.content);
        }

        f.close();
        ctx.showToast(QStringLiteral("📄 已导出: %1").arg(path));
        return true;
    }
};
