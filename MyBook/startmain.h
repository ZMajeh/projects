#ifndef STARTMAIN_H
#define STARTMAIN_H

#include <QMainWindow>
#include<QDir>
#include<QFile>
#include<QVBoxLayout>
#include<QtCore>
#include<QPushButton>
#include<QWidget>
#include<QGraphicsPixmapItem>
#include<QGraphicsView>
#include<QDebug>
#include<bookviewer.h>

namespace Ui {
    class startMain;
}

class startMain : public QMainWindow
{
    Q_OBJECT

public:
    explicit startMain(QWidget *parent = 0);
    ~startMain();

    QWidget* getBook(QString path);
    QWidget* addAdder();
    void init();
    void cleanUI();
    void readBooks(QGridLayout * layout);
    void addBooks();

private:
    Ui::startMain *ui;
    int colSize;
    int flag;
    QString bookTitle,bookPath;

private slots:
    void on_actionDecrease_row_size_triggered();
    void on_actionIncrease_row_size_triggered();
public Q_SLOTS:
    void on_adder_click();
    void on_book_click();
};

#endif // STARTMAIN_H
