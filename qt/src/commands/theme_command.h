#pragma once
#include "../command_plugin.h"
#include "../theme.h"

class ThemeCommand : public CommandPlugin {
public:
    QString name() const override { return QStringLiteral("theme"); }
    QString description() const override { return QStringLiteral("切换暗色/亮色主题"); }
    bool execute(const QString&, CommandContext& ctx) override {
        ThemeManager::instance().toggle();
        ctx.showToast(ThemeManager::instance().isDark()
            ? QStringLiteral("\xF0\x9F\x8C\x99 已切换到暗色主题")
            : QStringLiteral("\xE2\x98\x80\xEF\xB8\x8F 已切换到亮色主题"));
        return true;
    }
};
