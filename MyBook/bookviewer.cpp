#include "bookviewer.h"
#include "ui_bookviewer.h"


void BookViewer::setPath(QString tmp)
{
    path=tmp;
    qDebug()<<path<<" Receieved";
}

QWidget* BookViewer::getBook(QString path)
{
    QWidget *out=new QWidget();
    QVBoxLayout *out1=new QVBoxLayout();
    QPushButton *b1;
    b1=new QPushButton("Button 1");
    QString url=path;
    b1->setStyleSheet("background-image:url('"+url+"');");
    b1->setMinimumHeight(400);
    b1->setText(path.split("/")[path.split("/").length()-1]);
    connect(b1,SIGNAL(clicked()),this,SLOT(on_page_click()));
    out1->addWidget(b1);
    out->setLayout(out1);
    return out;
}

void BookViewer::cleanUI()
{
    QLayoutItem *child;
    while (ui->scrollAreaWidgetContents->layout()!=NULL && (child = ui->scrollAreaWidgetContents->layout()->takeAt(0)) != 0) {
        if(child->widget())
            child->widget()->setParent(NULL);
        delete child;
    }
    if(ui->scrollAreaWidgetContents->layout()!=NULL)
        delete ui->scrollAreaWidgetContents->layout();
}

void BookViewer::readPages(QGridLayout * layout)
{
    int i=0;
    QDir *dir=new QDir("./");
    if(!dir->exists("Books"))
        dir->mkdir("Books");
    else
    {
        dir->cd(path);
        qDebug()<<path<<" is getting loaded";

        foreach(QFileInfo file,dir->entryInfoList())
        {
            if(i++<2)continue;
            qDebug()<<file.absoluteFilePath();
            layout->addWidget(getBook(file.absoluteFilePath()));
        }
    }
}

void BookViewer::addBook()
{
    QGridLayout *layout=new QGridLayout();
    ui->scrollAreaWidgetContents->setLayout(layout);
    layout->setColumnMinimumWidth(colSize,5);
    layout->setVerticalSpacing(20);
    layout->setHorizontalSpacing(20);
    readPages(layout);
    layout->addWidget(addAdder());
    layout->addWidget(addRemover());
    for(int i=0;i<layout->rowCount();i++)
        layout->setRowMinimumHeight(i,400);
}

QWidget* BookViewer::addAdder()
{
    QWidget *out=new QWidget();
    QVBoxLayout *out1=new QVBoxLayout();
    QPushButton *b1;
    b1=new QPushButton("+");
    b1->setMinimumHeight(400);
    out1->addWidget(b1);
    out->setLayout(out1);
    connect(b1,SIGNAL(clicked()),this,SLOT(on_adder_click()));
    return out;
}

QWidget* BookViewer::addRemover()
{
    QWidget *out=new QWidget();
    QVBoxLayout *out1=new QVBoxLayout();
    QPushButton *b1;
    b1=new QPushButton("Delete");
    b1->setMinimumHeight(400);
    out1->addWidget(b1);
    out->setLayout(out1);
    connect(b1,SIGNAL(clicked()),this,SLOT(on_remover_click()));
    return out;
}

void BookViewer::init()
{
    //setCentralWidget(ui->scrollArea);
    cleanUI();
    addBook();
}



BookViewer::BookViewer(QWidget *parent) :
    QDialog(parent),
    ui(new Ui::BookViewer)
{
    ui->setupUi(this);
    colSize=1;
}

BookViewer::~BookViewer()
{
    delete ui;
}



void BookViewer::on_adder_click()
{
    QDir *dir=new QDir(path);
    int num=dir->count()-1;
    char buff[100];
    QString name="P"+QString(itoa(num,buff,10))+".png";
    QString finalPath=path+"/"+name;
    finalPath.replace("/","\\");
    QString cmd="copy \"Source\\template.png\" \""+finalPath+"\" || timeout 5";
    qDebug()<<finalPath<<" is getting created\n"<<cmd;
    system(cmd.toAscii());
    if(dir->exists(finalPath))
        init();
    else qDebug()<<finalPath<<" is invalid file directory";
}

void BookViewer::on_remover_click()
{
    QString cmd="dir&&move \""+path+"\" Deleted\\ || timeout 5";
    QString fileName=path.split("/")[path.split("/").length()-1];
    QString finalPath="Deleted\\"+fileName,cmd1="rd /s /q \""+finalPath+"\" || timeout 5";
    if(QDir(".\\"+finalPath).exists())
        system(cmd1.toAscii());
    qDebug()<<QDir(".\\"+finalPath).exists()<<finalPath<<" : "<<cmd<<"Deleting";
    system(cmd.toAscii());
    this->close();
}

void BookViewer::on_page_click()
{
    //Retrive sender button's name
    QPushButton* buttonSender = qobject_cast<QPushButton*>(sender()); // retrieve the button you have clicked
    QString page = buttonSender->text();
    QString finalPath=path+"\\"+page;
    QString cmd1 = "mspaint \""+finalPath+"\"";
    qDebug()<<cmd1<<":Opening paint";
    system(cmd1.toAscii());
}
