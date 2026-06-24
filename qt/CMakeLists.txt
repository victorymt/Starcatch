cmake_minimum_required(VERSION 3.16)
project(StarcatchQt VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Sql)

set(SOURCES
    src/main.cpp
    src/allpanel.cpp
    src/database.cpp
    src/command_plugin.cpp
    src/inputparser.cpp
    src/mainwindow.cpp
    src/theme.cpp
    src/todopanel.cpp
    src/ideapanel.cpp
    src/logpanel.cpp
    src/quickinputbar.cpp
    src/toastwidget.cpp
)

set(HEADERS
    src/allpanel.h
    src/models.h
    src/command_plugin.h
    src/database.h
    src/inputparser.h
    src/mainwindow.h
    src/todopanel.h
    src/ideapanel.h
    src/logpanel.h
    src/quickinputbar.h
    src/toastwidget.h
    src/theme.h
    src/commands/help_command.h
    src/commands/theme_command.h
    src/commands/search_command.h
    src/commands/export_command.h
    src/commands/test_delete_command.h
)

add_executable(starcatch-qt ${SOURCES} ${HEADERS})

target_link_libraries(starcatch-qt PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Widgets
    Qt6::Sql
)
