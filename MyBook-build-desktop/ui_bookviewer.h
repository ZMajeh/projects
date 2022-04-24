/********************************************************************************
** Form generated from reading UI file 'bookviewer.ui'
**
** Created: Sun 24. Apr 16:05:06 2022
**      by: Qt User Interface Compiler version 4.7.0
**
** WARNING! All changes made in this file will be lost when recompiling UI file!
********************************************************************************/

#ifndef UI_BOOKVIEWER_H
#define UI_BOOKVIEWER_H

#include <QtCore/QVariant>
#include <QtGui/QAction>
#include <QtGui/QApplication>
#include <QtGui/QButtonGroup>
#include <QtGui/QDialog>
#include <QtGui/QHeaderView>
#include <QtGui/QScrollArea>
#include <QtGui/QVBoxLayout>
#include <QtGui/QWidget>

QT_BEGIN_NAMESPACE

class Ui_BookViewer
{
public:
    QVBoxLayout *verticalLayout;
    QScrollArea *scrollArea;
    QWidget *scrollAreaWidgetContents;

    void setupUi(QDialog *BookViewer)
    {
        if (BookViewer->objectName().isEmpty())
            BookViewer->setObjectName(QString::fromUtf8("BookViewer"));
        BookViewer->resize(401, 289);
        verticalLayout = new QVBoxLayout(BookViewer);
        verticalLayout->setObjectName(QString::fromUtf8("verticalLayout"));
        scrollArea = new QScrollArea(BookViewer);
        scrollArea->setObjectName(QString::fromUtf8("scrollArea"));
        scrollArea->setWidgetResizable(true);
        scrollAreaWidgetContents = new QWidget();
        scrollAreaWidgetContents->setObjectName(QString::fromUtf8("scrollAreaWidgetContents"));
        scrollAreaWidgetContents->setGeometry(QRect(0, 0, 375, 263));
        scrollArea->setWidget(scrollAreaWidgetContents);

        verticalLayout->addWidget(scrollArea);


        retranslateUi(BookViewer);

        QMetaObject::connectSlotsByName(BookViewer);
    } // setupUi

    void retranslateUi(QDialog *BookViewer)
    {
        BookViewer->setWindowTitle(QApplication::translate("BookViewer", "Dialog", 0, QApplication::UnicodeUTF8));
    } // retranslateUi

};

namespace Ui {
    class BookViewer: public Ui_BookViewer {};
} // namespace Ui

QT_END_NAMESPACE

#endif // UI_BOOKVIEWER_H
