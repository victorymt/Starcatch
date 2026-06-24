#pragma once
#include "../command_plugin.h"

class SearchCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("search"); }
    QString description() const override { return QStringLiteral("在 All 视图搜索所有条目"); }
    QString usage() const override { return QStringLiteral("<关键词>"); }

    bool execute(const QString& args, CommandContext& ctx) override {
        if (args.isEmpty()) {
            ctx.showToast(QStringLiteral("用法: /search <关键词>"));
            return true;
        }
        ctx.searchInAll(args);
        return true;
    }
};
