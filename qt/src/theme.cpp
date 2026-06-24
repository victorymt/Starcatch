#include "theme.h"
#include <QDir>
#include <QFile>
#include <QTextStream>

ThemeManager::ThemeManager() {
    const QString dir = QDir::homePath() + QStringLiteral("/.local/share/starcatch");
    QDir().mkpath(dir);
    m_configPath = dir + QStringLiteral("/theme");
    QFile f(m_configPath);
    if (f.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QString val = QString::fromUtf8(f.readAll()).trimmed();
        m_dark = (val != QStringLiteral("light"));
    }
}

ThemeManager& ThemeManager::instance() {
    static ThemeManager mgr;
    return mgr;
}

void ThemeManager::toggle() { setDark(!m_dark); }
void ThemeManager::setDark(bool dark) {
    m_dark = dark;
    QFile f(m_configPath);
    if (f.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) {
        QTextStream out(&f);
        out << (m_dark ? "dark" : "light");
    }
    apply(qApp);
}

void ThemeManager::apply(QApplication* app) {
    app->setStyleSheet(QString());
    app->setStyleSheet(baseStyles() + (m_dark ? darkStyles() : lightStyles()));
}

QString ThemeManager::baseStyles() const {
    return QStringLiteral(
        "* { font-family: \"Noto Sans\", \"Noto Sans CJK SC\", sans-serif; }"
        "QScrollArea { border: none; background: transparent; }"
        "QScrollBar:vertical { width: 6px; background: transparent; }"
        "QScrollBar::handle:vertical { border-radius: 3px; min-height: 20px; }"
        "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0; }"
        "QTabWidget::pane { border: none; }"
        "QTabBar::tab { padding: 8px 18px; border-radius: 10px; margin: 3px 3px; font-size: 13px; }"
        "QPushButton { border-radius: 6px; padding: 5px 14px; font-size: 12px; border: 1px solid transparent; }"
        "QPushButton:checked { font-weight: bold; }"
        "QToolButton { border-radius: 4px; padding: 2px 6px; border: none; font-size: 14px; }"
        "QLineEdit { border-radius: 10px; padding: 8px 14px; font-size: 13px; border: 1px solid; }"
        "QComboBox { border-radius: 6px; padding: 5px 10px; font-size: 13px; }"
        "QComboBox::drop-down { border: none; }"
        "QSlider::groove:horizontal { height: 4px; border-radius: 2px; }"
        "QSlider::handle:horizontal { width: 14px; height: 14px; margin: -5px 0; border-radius: 7px; }"
    );
}

QString ThemeManager::darkStyles() const {
    return QStringLiteral(
        "QWidget { background-color: #1a1a2e; color: #e0e0e0; }"
        "QTabBar::tab { background: #222240; color: #888; }"
        "QTabBar::tab:selected { background: #16213e; color: #64b5f6; }"
        "QPushButton { background: #2a2a4a; color: #ccc; border-color: #333; }"
        "QPushButton:hover { background: #333360; }"
        "QPushButton:checked { background: #16213e; color: #64b5f6; border-color: #64b5f6; }"
        "QPushButton:pressed { background: #1a1a3a; }"
        "QToolButton { background: transparent; color: #888; }"
        "QToolButton:hover { background: #333360; }"
        "QLineEdit { background: #222240; color: #e0e0e0; border-color: #333; }"
        "QLineEdit:focus { border-color: #64b5f6; }"
        "QComboBox { background: #222240; color: #e0e0e0; border: 1px solid #333; }"
        "QComboBox QAbstractItemView { background: #222240; selection-background-color: #16213e; border-radius: 6px; }"
        "QCheckBox { color: #ccc; spacing: 6px; }"
        "QCheckBox::indicator { width: 16px; height: 16px; border-radius: 3px;"
        "  border: 2px solid #666; background: transparent; }"
        "QCheckBox::indicator:checked { background: #64b5f6; border-color: #64b5f6; }"
        "QCheckBox::indicator:hover { border-color: #90caf9; }"
        "QScrollBar::handle:vertical { background: #444; }"
        "QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical { background: transparent; }"
        "QSlider::groove:horizontal { background: #333; }"
        "QSlider::handle:horizontal { background: #64b5f6; }"
        "QMessageBox { background-color: #1a1a2e; }"
        "QMessageBox QLabel { color: #e0e0e0; }"
        "QFrame[card=\"true\"] { background: #222240; border-radius: 8px; border: 1px solid #2a2a4a; padding: 2px; margin: 1px 4px; }"
        "QFrame[card=\"true\"] QLabel { background: transparent; }"
        "QFrame[card=\"true\"]:hover { background: #252548; border-color: #3a3a5a; }"
    );
}

QString ThemeManager::lightStyles() const {
    return QStringLiteral(
        "QWidget { background-color: #f5f5f5; color: #333; }"
        "QTabBar::tab { background: #e0e0e0; color: #666; }"
        "QTabBar::tab:selected { background: #fff; color: #1565c0; }"
        "QPushButton { background: #e8e8e8; color: #444; border-color: #ccc; }"
        "QPushButton:hover { background: #ddd; }"
        "QPushButton:checked { background: #bbdefb; color: #1565c0; border-color: #1565c0; }"
        "QPushButton:pressed { background: #ccc; }"
        "QToolButton { background: transparent; color: #888; }"
        "QToolButton:hover { background: #ddd; }"
        "QLineEdit { background: #fff; color: #333; border-color: #ccc; }"
        "QLineEdit:focus { border-color: #1565c0; }"
        "QComboBox { background: #fff; color: #333; border: 1px solid #ccc; }"
        "QComboBox QAbstractItemView { background: #fff; selection-background-color: #bbdefb; border-radius: 6px; }"
        "QCheckBox { color: #444; spacing: 6px; }"
        "QScrollBar::handle:vertical { background: #bbb; }"
        "QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical { background: transparent; }"
        "QSlider::groove:horizontal { background: #ddd; }"
        "QSlider::handle:horizontal { background: #1565c0; }"
        "QMessageBox { background-color: #f5f5f5; }"
        "QMessageBox QLabel { color: #333; }"
        "QFrame[card=\"true\"] { background: #fff; border-radius: 8px; border: 1px solid #e0e0e0; padding: 2px; margin: 1px 4px; }"
        "QFrame[card=\"true\"] QLabel { background: transparent; }"
        "QFrame[card=\"true\"]:hover { background: #fafafa; border-color: #bbb; }"
    );
}
