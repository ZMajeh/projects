/********************************************************************************
** Form generated from reading UI file 'startmain.ui'
**
** Created by: Qt User Interface Compiler version 4.8.7
**
** WARNING! All changes made in this file will be lost when recompiling UI file!
********************************************************************************/

#ifndef UI_STARTMAIN_H
#define UI_STARTMAIN_H

#include <QtCore/QVariant>
#include <QtGui/QAction>
#include <QtGui/QApplication>
#include <QtGui/QButtonGroup>
#include <QtGui/QHeaderView>
#include <QtGui/QMainWindow>
#include <QtGui/QMenu>
#include <QtGui/QMenuBar>
#include <QtGui/QScrollArea>
#include <QtGui/QStatusBar>
#include <QtGui/QToolBar>
#include <QtGui/QWidget>

QT_BEGIN_NAMESPACE

class Ui_startMain
{
public:
    QAction *actionNew;
    QAction *actionOpen;
    QAction *actionExit;
    QAction *actionAbout_us;
    QAction *actionIncrease_row_size;
    QAction *actionDecrease_row_size;
    QWidget *centralWidget;
    QScrollArea *scrollArea;
    QWidget *scrollAreaWidgetContents;
    QMenuBar *menuBar;
    QMenu *menuFile;
    QMenu *menuHelp;
    QMenu *menuTools;
    QMenu *menuView;
    QMenu *menuEdit;
    QToolBar *mainToolBar;
    QStatusBar *statusBar;

    void setupUi(QMainWindow *startMain)
    {
        if (startMain->objectName().isEmpty())
            startMain->setObjectName(QString::fromUtf8("startMain"));
        startMain->resize(400, 300);
        actionNew = new QAction(startMain);
        actionNew->setObjectName(QString::fromUtf8("actionNew"));
        actionOpen = new QAction(startMain);
        actionOpen->setObjectName(QString::fromUtf8("actionOpen"));
        actionExit = new QAction(startMain);
        actionExit->setObjectName(QString::fromUtf8("actionExit"));
        actionAbout_us = new QAction(startMain);
        actionAbout_us->setObjectName(QString::fromUtf8("actionAbout_us"));
        actionIncrease_row_size = new QAction(startMain);
        actionIncrease_row_size->setObjectName(QString::fromUtf8("actionIncrease_row_size"));
        actionDecrease_row_size = new QAction(startMain);
        actionDecrease_row_size->setObjectName(QString::fromUtf8("actionDecrease_row_size"));
        centralWidget = new QWidget(startMain);
        centralWidget->setObjectName(QString::fromUtf8("centralWidget"));
        scrollArea = new QScrollArea(centralWidget);
        scrollArea->setObjectName(QString::fromUtf8("scrollArea"));
        scrollArea->setGeometry(QRect(10, 10, 361, 241));
        scrollArea->setWidgetResizable(true);
        scrollAreaWidgetContents = new QWidget();
        scrollAreaWidgetContents->setObjectName(QString::fromUtf8("scrollAreaWidgetContents"));
        scrollAreaWidgetContents->setGeometry(QRect(0, 0, 357, 237));
        scrollArea->setWidget(scrollAreaWidgetContents);
        startMain->setCentralWidget(centralWidget);
        menuBar = new QMenuBar(startMain);
        menuBar->setObjectName(QString::fromUtf8("menuBar"));
        menuBar->setGeometry(QRect(0, 0, 400, 20));
        menuFile = new QMenu(menuBar);
        menuFile->setObjectName(QString::fromUtf8("menuFile"));
        menuHelp = new QMenu(menuBar);
        menuHelp->setObjectName(QString::fromUtf8("menuHelp"));
        menuTools = new QMenu(menuBar);
        menuTools->setObjectName(QString::fromUtf8("menuTools"));
        menuView = new QMenu(menuBar);
        menuView->setObjectName(QString::fromUtf8("menuView"));
        menuEdit = new QMenu(menuBar);
        menuEdit->setObjectName(QString::fromUtf8("menuEdit"));
        startMain->setMenuBar(menuBar);
        mainToolBar = new QToolBar(startMain);
        mainToolBar->setObjectName(QString::fromUtf8("mainToolBar"));
        startMain->addToolBar(Qt::TopToolBarArea, mainToolBar);
        statusBar = new QStatusBar(startMain);
        statusBar->setObjectName(QString::fromUtf8("statusBar"));
        startMain->setStatusBar(statusBar);

        menuBar->addAction(menuFile->menuAction());
        menuBar->addAction(menuEdit->menuAction());
        menuBar->addAction(menuView->menuAction());
        menuBar->addAction(menuTools->menuAction());
        menuBar->addAction(menuHelp->menuAction());
        menuFile->addAction(actionNew);
        menuFile->addAction(actionOpen);
        menuFile->addAction(actionExit);
        menuHelp->addAction(actionAbout_us);
        menuView->addAction(actionIncrease_row_size);
        menuView->addAction(actionDecrease_row_size);

        retranslateUi(startMain);

        QMetaObject::connectSlotsByName(startMain);
    } // setupUi

    void retranslateUi(QMainWindow *startMain)
    {
        startMain->setWindowTitle(QApplication::translate("startMain", "startMain", 0, QApplication::UnicodeUTF8));
        actionNew->setText(QApplication::translate("startMain", "New", 0, QApplication::UnicodeUTF8));
        actionOpen->setText(QApplication::translate("startMain", "Open", 0, QApplication::UnicodeUTF8));
        actionExit->setText(QApplication::translate("startMain", "Exit", 0, QApplication::UnicodeUTF8));
        actionAbout_us->setText(QApplication::translate("startMain", "About us", 0, QApplication::UnicodeUTF8));
        actionIncrease_row_size->setText(QApplication::translate("startMain", "Increase row size", 0, QApplication::UnicodeUTF8));
        actionDecrease_row_size->setText(QApplication::translate("startMain", "Decrease row size", 0, QApplication::UnicodeUTF8));
        menuFile->setTitle(QApplication::translate("startMain", "File", 0, QApplication::UnicodeUTF8));
        menuHelp->setTitle(QApplication::translate("startMain", "Help", 0, QApplication::UnicodeUTF8));
        menuTools->setTitle(QApplication::translate("startMain", "Tools", 0, QApplication::UnicodeUTF8));
        menuView->setTitle(QApplication::translate("startMain", "View", 0, QApplication::UnicodeUTF8));
        menuEdit->setTitle(QApplication::translate("startMain", "Edit", 0, QApplication::UnicodeUTF8));
    } // retranslateUi

};

namespace Ui {
    class startMain: public Ui_startMain {};
} // namespace Ui

QT_END_NAMESPACE

#endif // UI_STARTMAIN_H
