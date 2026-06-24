#pragma once

#include "../command_plugin.h"
#include <QMessageBox>

/// /help — display all registered commands in a dialog.
class HelpCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("help"); }
    QString description() const override { return QStringLiteral("显示所有可用命令"); }

    bool execute(const QString& args, CommandContext& ctx) override {
        Q_UNUSED(args);

        QStringList lines;
        lines << QStringLiteral("快速切换：\n"
            "  /t [内容]    切换到 Todo 输入\n"
            "  /i [内容]    切换到 Idea 输入\n"
            "  /l [内容]    切换到 Log 输入\n");

        lines << QStringLiteral("命令：");

        for (auto* p : CommandRegistry::instance().all()) {
            QString line = QStringLiteral("  /%1").arg(p->name());
            if (!p->usage().isEmpty()) {
                line += QStringLiteral("  %1").arg(p->usage());
            }
            line += QStringLiteral("    — %1").arg(p->description());
            lines << line;
        }

        lines << QStringLiteral("\n快捷键：\n"
            "  Enter  提交\n"
            "  Esc    关闭窗口");

        QMessageBox::information(ctx.parentWindow,
            QStringLiteral("Starcatch 命令"),
            lines.join(QStringLiteral("\n")));

        ctx.inputBar->focusInput();
        return true; // clear input
    }
};
