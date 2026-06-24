#pragma once
#include "../command_plugin.h"

class SearchCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("search"); }
    QString description() const override { return QStringLiteral("搜索所有条目"); }
    QString usage() const override { return QStringLiteral("<关键词>"); }

    bool execute(const QString& args, CommandContext& ctx) override {
        if (args.isEmpty()) {
            ctx.showToast(QStringLiteral("用法: /search <关键词>"));
            return true;
        }

        QStringList results;
        QString query = args.toLower();

        // Search Todos
        auto todos = ctx.db->listTodosByStatuses(
            {QStringLiteral("pending"), QStringLiteral("done"), QStringLiteral("archived")});
        for (const auto& t : todos) {
            if (t.title.toLower().contains(query) ||
                t.tags.filter(query, Qt::CaseInsensitive).size() > 0 ||
                t.project.toLower().contains(query)) {
                results << QStringLiteral("📋 %1 %2")
                    .arg(priorityToString(t.priority), t.title);
            }
        }

        // Search Ideas
        auto ideas = ctx.db->listIdeas(365);
        for (const auto& i : ideas) {
            if (i.title.toLower().contains(query) ||
                i.content.toLower().contains(query)) {
                results << QStringLiteral("💡 %1").arg(i.title);
            }
        }

        // Search Logs
        auto logs = ctx.db->listLogs(365);
        for (const auto& l : logs) {
            if (l.content.toLower().contains(query)) {
                QString preview = l.content.left(80);
                if (l.content.size() > 80) preview += QStringLiteral("...");
                results << QStringLiteral("📝 %1").arg(preview);
            }
        }

        if (results.isEmpty()) {
            ctx.showToast(QStringLiteral("🔍 未找到 \"%1\"").arg(args));
        } else {
            ctx.showToast(QStringLiteral("🔍 找到 %1 条结果").arg(results.size()));
            // Show first 10 results in a dialog
            QStringList shown = results.mid(0, 10);
            if (results.size() > 10)
                shown << QStringLiteral("... 还有 %1 条").arg(results.size() - 10);
            QMessageBox::information(ctx.parentWindow,
                QStringLiteral("搜索: %1").arg(args),
                shown.join(QStringLiteral("\n")));
        }
        return true;
    }
};
