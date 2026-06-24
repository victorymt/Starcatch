#pragma once

#include <QString>
#include <QApplication>

/// Manages light/dark theme via global QSS stylesheet.
/// Persists preference to ~/.local/share/starcatch/theme.
class ThemeManager {
public:
    static ThemeManager& instance();

    bool isDark() const { return m_dark; }
    void toggle();
    void setDark(bool dark);
    void apply(QApplication* app);

private:
    ThemeManager();
    void save();
    QString stylesheet() const;
    QString baseStyles() const;
    QString darkStyles() const;
    QString lightStyles() const;

    bool m_dark = true;
    QString m_configPath;
};
