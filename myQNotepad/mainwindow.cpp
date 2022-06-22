#include "mainwindow.h"
#include "ui_mainwindow.h"

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    this->setCentralWidget(ui->textEdit);
    this->setWindowTitle("Untitled");
    isNewFile=true;
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::on_actionNew_triggered()
{
    this->setWindowTitle("Untitled");
    ui->textEdit->setText("");
    ui->textEdit->clear();
    isNewFile=true;
}

void MainWindow::on_actionOpen_triggered()
{
    myFile=new QFile(QFileDialog::getOpenFileName(this,"Select file to open"));
    if(myFile->open(QIODevice::ReadOnly | QIODevice::Text))
    {
        QTextStream myStream(myFile);
        ui->textEdit->setText(myStream.readAll());
        myFile->close();

        this->setWindowTitle(myFile->fileName());
        isNewFile=false;
    }
}

void MainWindow::on_actionClose_triggered()
{
    exit(0);
}

void MainWindow::on_actionSave_triggered()
{
    if(this->windowTitle()=="Untitled" && isNewFile)
        myFile=new QFile(QFileDialog::getSaveFileName(this,"Select file to save"));
    if(myFile->open(QIODevice::WriteOnly | QIODevice::Text))
    {
        QTextStream myStream(myFile);
        myStream << ui->textEdit->toPlainText();
        myStream.flush();
        myFile->flush();
        myFile->close();

        this->setWindowTitle(myFile->fileName());
        isNewFile=false;
    }
    else
    {
        QMessageBox::warning(this,"Warning","Could not save the file : "+myFile->fileName());
    }
}

void MainWindow::on_actionSave_as_triggered()
{
    myFile=new QFile(QFileDialog::getSaveFileName(this,"Select file to save as"));
    if(myFile->open(QIODevice::WriteOnly | QIODevice::Text))
    {
        QTextStream myStream(myFile);
        myStream << ui->textEdit->toPlainText();
        myStream.flush();
        myFile->flush();
        myFile->close();

        this->setWindowTitle(myFile->fileName());
        isNewFile=false;
    }
}

void MainWindow::on_actionAbout_us_triggered()
{
    QMessageBox::information(this,"H&M production","Hydra and Majeh production. Always create someting new. Contact me on ZMajeh@gmail.com.");
}

void MainWindow::on_actionUndo_triggered()
{
    ui->textEdit->undo();
}

void MainWindow::on_actionRedo_triggered()
{
    ui->textEdit->redo();
}

void MainWindow::on_actionCopy_triggered()
{
    ui->textEdit->copy();
}

void MainWindow::on_actionPaste_triggered()
{
    ui->textEdit->paste();
}

void MainWindow::on_actionCut_triggered()
{
    ui->textEdit->cut();
}

void MainWindow::on_actionDelete_triggered()
{
    ui->textEdit->cut();
    ui->textEdit->cut();
}
