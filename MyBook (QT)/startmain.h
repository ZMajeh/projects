#ifndef STARTMAIN_H
#define STARTMAIN_H

#include <QMainWindow>
#include <QToolButton>
#include <QProgressDialog>
#include <QThread>
#include <QPrinter>
#include <QPainter>
#include <QDir>
#include <QFileInfoList>
#include <QMessageBox>
#include <QFileDialog>
#include <QImage>
#include <QBuffer>
#include<utils.h>


class ExportWorker : public QThread {
    Q_OBJECT
public:
    ExportWorker(QString dirPath, QString savePath, QObject *parent = 0);

signals:
    void progress(int value);
    void finished(QString pdfPath);
    void error(QString message);

protected:
    void run();

private:
    QString m_dirPath;
    QString m_savePath;
};

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
    void exportDirToPdf(QString dirPath);

private:
    Ui::startMain *ui;
    int colSize;
    int flag;
    QString bookTitle,bookPath;

private slots:
    void on_actionDecrease_row_size_triggered();
    void on_actionIncrease_row_size_triggered();
public Q_SLOTS:
    void adderClicked();
    void bookClicked();

private slots:
    void onDownloadBtnClicked();
    void onExportProgress(int value);
    void onExportFinished(QString pdfPath);
    void onExportError(QString message);

private:
    QToolButton *downloadBtn;
    QProgressDialog *progressDialog;

};

#endif // STARTMAIN_H
