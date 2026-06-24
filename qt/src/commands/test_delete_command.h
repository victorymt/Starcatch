#pragma once
#include "../command_plugin.h"
#include <QSqlQuery>
#include <QSqlError>
#include <QMessageBox>

class TestDeleteAllCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("test-delete-all"); }
    QString description() const override { return QStringLiteral("[测试] 删除所有条目"); }

    bool execute(const QString&, CommandContext& ctx) override {
        auto answer = QMessageBox::warning(ctx.parentWindow,
            QStringLiteral("危险操作"),
            QStringLiteral("确定要删除所有条目吗？此操作不可撤销！"),
            QMessageBox::Yes | QMessageBox::No, QMessageBox::No);

        if (answer != QMessageBox::Yes) {
            ctx.showToast(QStringLiteral("已取消"));
            return true;
        }

        QSqlDatabase db = QSqlDatabase::database(QStringLiteral("starcatch_conn"));
        QSqlQuery q(db);

        int total = 0;
        q.exec(QStringLiteral("SELECT COUNT(*) FROM todos"));
        if (q.next()) total += q.value(0).toInt();
        q.exec(QStringLiteral("SELECT COUNT(*) FROM ideas"));
        if (q.next()) total += q.value(0).toInt();
        q.exec(QStringLiteral("SELECT COUNT(*) FROM logs"));
        if (q.next()) total += q.value(0).toInt();

        q.exec(QStringLiteral("DELETE FROM todos"));
        q.exec(QStringLiteral("DELETE FROM ideas"));
        q.exec(QStringLiteral("DELETE FROM logs"));

        ctx.refreshCurrentPanel();
        ctx.showToast(QStringLiteral("已删除 %1 条条目").arg(total));
        return true;
    }
};
