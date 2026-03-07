#ifndef BOOKVIEWER_H
#define BOOKVIEWER_H

#include<utils.h>
namespace Ui {
    class BookViewer;
}

class BookViewer : public QDialog
{
    Q_OBJECT

public:
    explicit BookViewer(QWidget *parent = 0);
    ~BookViewer();
    void setPath(QString tmp);
    QWidget* getBook(QString path);
    QWidget* addAdder();
    void init();
    void cleanUI();
    void readPages(QGridLayout * layout);
    void addBook();
    QWidget* addRemover();


private:
    int colSize;
    QString path;
    Ui::BookViewer *ui;

public Q_SLOTS:
    void on_adder_click();
    void on_remover_click();
    void on_page_click();
};

#endif // BOOKVIEWER_H
