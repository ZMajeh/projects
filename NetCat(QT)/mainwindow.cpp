#include "mainwindow.h"
#include "ui_mainwindow.h"

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    socket=new QUdpSocket(this);
}

MainWindow::~MainWindow()
{
    delete ui;
}

// Exit event
void MainWindow::on_actionExit_triggered()
{
    exit(1);
}
// When clicked send
void MainWindow::on_pushButton_clicked()
{
    // Get hostaddress, port and msg to generate datagram
    QString host,msg;
    quint16 port;
    host=ui->lineEdit->text();
    port=ui->lineEdit_2->text().toInt();
    msg=ui->textEdit->toPlainText();
    // This is buffer which we write
    QByteArray qba;
    qba.append(msg);
    // To send N number of datagram packets
    long int n=ui->lineEdit_3->text().toInt();
    if(n>0)// If n is valid positive number
        while(n-->0) // Send datagram n times
            socket->writeDatagram(qba,QHostAddress(host),port);
    else// Else send only once
        socket->writeDatagram(qba,QHostAddress(host),port);
}
