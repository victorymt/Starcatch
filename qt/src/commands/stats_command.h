#pragma once
#include "../command_plugin.h"
#include <QSqlQuery>
#include <QMessageBox>

class StatsCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("stats"); }
    QString description() const override { return QStringLiteral("显示统计数据"); }

    bool execute(const QString&, CommandContext& ctx) override {
        QSqlDatabase db = QSqlDatabase::database(QStringLiteral("starcatch_conn"));
        QSqlQuery q(db);

        auto count = [&](const QString& sql) -> int {
            q.exec(sql);
            return q.next() ? q.value(0).toInt() : 0;
        };

        int pending  = count(QStringLiteral("SELECT COUNT(*) FROM todos WHERE status='pending'"));
        int done     = count(QStringLiteral("SELECT COUNT(*) FROM todos WHERE status='done'"));
        int archived = count(QStringLiteral("SELECT COUNT(*) FROM todos WHERE status='archived'"));
        int ideas    = count(QStringLiteral("SELECT COUNT(*) FROM ideas"));
        int logs     = count(QStringLiteral("SELECT COUNT(*) FROM logs"));
        int totalTodos = pending + done + archived;

        double rate = totalTodos > 0 ? (double)done / totalTodos * 100.0 : 0;

        QString msg;
        msg += QStringLiteral("📋 Todo\n");
        msg += QStringLiteral("  待办: %1  已完成: %2  归档: %3  合计: %4\n")
            .arg(pending).arg(done).arg(archived).arg(totalTodos);
        msg += QStringLiteral("  完成率: %1%\n\n").arg(rate, 0, 'f', 0);

        msg += QStringLiteral("💭 Idea: %1\n").arg(ideas);
        msg += QStringLiteral("📓 Log:  %1\n").arg(logs);
        msg += QStringLiteral("\n📊 总计: %1 条").arg(totalTodos + ideas + logs);

        QMessageBox::information(ctx.parentWindow, QStringLiteral("Starcatch 统计"), msg);
        return true;
    }
};
