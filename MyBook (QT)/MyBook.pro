#-------------------------------------------------
#
# Project created by QtCreator 2022-04-24T10:55:54
#
#-------------------------------------------------

QT       += core gui

TARGET = MyBook
TEMPLATE = app


SOURCES += main.cpp\
        startmain.cpp \
    bookviewer.cpp \
    utils.cpp

HEADERS  += startmain.h \
    bookviewer.h \
    utils.h

FORMS    += startmain.ui \
    bookviewer.ui

CONFIG += console
CONFIG += release
