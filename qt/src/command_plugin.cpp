#include "command_plugin.h"

CommandRegistry& CommandRegistry::instance() {
    static CommandRegistry reg;
    return reg;
}

void CommandRegistry::registerPlugin(std::unique_ptr<CommandPlugin> plugin) {
    const QString key = plugin->name();
    auto* raw = plugin.get();
    m_plugins.emplace(key, std::move(plugin));
    m_ordered.push_back(raw);
}

CommandPlugin* CommandRegistry::find(const QString& name) {
    auto it = m_plugins.find(name);
    return (it != m_plugins.end()) ? it->second.get() : nullptr;
}

QVector<CommandPlugin*> CommandRegistry::all() const {
    QVector<CommandPlugin*> result;
    result.reserve(static_cast<int>(m_ordered.size()));
    for (auto* p : m_ordered) {
        result.append(p);
    }
    return result;
}
