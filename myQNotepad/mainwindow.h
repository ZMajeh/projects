#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QFile>
#include <QIODevice>
#include <QTextStream>
#include <QFileDialog>
#include <QMessageBox>

namespace Ui {
    class MainWindow;
}

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = 0);
    ~MainWindow();

private:
    Ui::MainWindow *ui;
    QFile *myFile;
    bool isNewFile;
private slots:
    void on_actionDelete_triggered();
    void on_actionCut_triggered();
    void on_actionPaste_triggered();
    void on_actionCopy_triggered();
    void on_actionRedo_triggered();
    void on_actionUndo_triggered();
    void on_actionAbout_us_triggered();
    void on_actionSave_as_triggered();
    void on_actionSave_triggered();
    void on_actionClose_triggered();
    void on_actionOpen_triggered();
    void on_actionNew_triggered();
};

#endif // MAINWINDOW_H
