#pragma once

#include <QString>
#include <QVector>
#include <QWidget>
#include <QHash>
#include <memory>
#include <functional>
#include <unordered_map>
#include <vector>

/// std::hash adapter so QString can be used as an unordered_map key.
struct QStringHash {
    std::size_t operator()(const QString& s) const noexcept {
        return static_cast<std::size_t>(qHash(s));
    }
};

class Database;
class QuickInputBar;

/// Context passed to every command's execute().
/// Provides access to the database, UI components, and convenience callbacks.
struct CommandContext {
    Database* db = nullptr;
    QWidget*  parentWindow = nullptr;
    QuickInputBar* inputBar = nullptr;

    /// Show a green toast message in the main window.
    std::function<void(const QString&)> showToast;

    /// Refresh the currently active tab panel.
    std::function<void()> refreshCurrentPanel;

    /// Switch to All tab and show search results for the given query.
    std::function<void(const QString&)> searchInAll;
};

/// Abstract base for a slash-command plugin.
///
/// Usage:
///   1. Subclass CommandPlugin, implement name(), description(), execute().
///   2. Register with CommandRegistry::instance().registerPlugin(...).
///
/// Kind-switch commands (/t /i /l) are NOT plugins — they live in QuickInputBar.
class CommandPlugin {
public:
    virtual ~CommandPlugin() = default;

    /// Command name without the leading slash.  E.g. "help", "search".
    virtual QString name() const = 0;

    /// One-line description shown in /help output.
    virtual QString description() const = 0;

    /// Optional usage hint.  E.g. "Usage: /search <keyword>"
    virtual QString usage() const { return QString(); }

    /// Execute the command.
    /// @param args  Everything after the command word (already trimmed).
    /// @param ctx   Access to DB, window, input bar, toast, refresh.
    /// @return true if the input bar should be cleared after execution.
    virtual bool execute(const QString& args, CommandContext& ctx) = 0;
};

/// Singleton registry of all CommandPlugin instances.
///
/// Plugins are registered once at startup and live for the app lifetime.
class CommandRegistry {
public:
    static CommandRegistry& instance();

    /// Take ownership of the plugin. Registered plugins appear in /help.
    void registerPlugin(std::unique_ptr<CommandPlugin> plugin);

    /// Look up a plugin by name. Returns nullptr if not found.
    CommandPlugin* find(const QString& name);

    /// All registered plugins, in registration order.
    QVector<CommandPlugin*> all() const;

    CommandRegistry(const CommandRegistry&) = delete;
    CommandRegistry& operator=(const CommandRegistry&) = delete;

private:
    CommandRegistry() = default;

    std::unordered_map<QString, std::unique_ptr<CommandPlugin>, QStringHash> m_plugins;
    std::vector<CommandPlugin*> m_ordered;
};

/// Convenience: register a plugin by type, forwarding constructor args.
template <typename T, typename... Args>
void registerCommand(Args&&... args) {
    CommandRegistry::instance().registerPlugin(
        std::make_unique<T>(std::forward<Args>(args)...));
}
