#include "startmain.h"
#include "ui_startmain.h"

QWidget* startMain::getBook(QString path)
{
    QWidget *out=new QWidget();
    QVBoxLayout *out1=new QVBoxLayout();
    QPushButton *b1;//,*b2;
    b1=new QPushButton("Button 1");
    QString url=path+"/p1.png";
    b1->setStyleSheet("background-image:url('"+url+"');");
    b1->setMinimumHeight(400);
    b1->setText(path);//.split("/")[path.split("/").length()-1]);
    connect(b1,SIGNAL(clicked()),this,SLOT(on_book_click()));
    out1->addWidget(b1);
    out->setLayout(out1);
    return out;
}

void startMain::cleanUI()
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

void startMain::readBooks(QGridLayout * layout)
{
    int i=0;
    QDir *dir=new QDir("./");
    if(!dir->exists("Books"))
        dir->mkdir("Books");
    if(!dir->exists("Deleted"))
        dir->mkdir("Deleted");
    else
    {
        dir->cd("Books");
        foreach(QFileInfo file,dir->entryInfoList())
        {
            if(i++<2)continue;
            qDebug()<<file.absoluteFilePath();
            layout->addWidget(getBook(file.absoluteFilePath()));
        }
    }
}

void startMain::addBooks()
{
    QGridLayout *layout=new QGridLayout();
    ui->scrollAreaWidgetContents->setLayout(layout);
    layout->setColumnMinimumWidth(colSize,5);
    layout->setVerticalSpacing(20);
    layout->setHorizontalSpacing(20);
    readBooks(layout);
    layout->addWidget(addAdder());
    for(int i=0;i<layout->rowCount();i++)
        layout->setRowMinimumHeight(i,400);
}

QWidget* startMain::addAdder()
{
    QWidget *out=new QWidget();
    QVBoxLayout *out1=new QVBoxLayout();
    QPushButton *b1;//,*b2;
    b1=new QPushButton("+");
    b1->setMinimumHeight(400);
    out1->addWidget(b1);
    out->setLayout(out1);
    connect(b1,SIGNAL(clicked()),this,SLOT(on_adder_click()));
    return out;
}

void startMain::init()
{
    setCentralWidget(ui->scrollArea);
    cleanUI();
    addBooks();
}

startMain::startMain(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::startMain)
{
    ui->setupUi(this);
    colSize=4;
    init();
}

startMain::~startMain()
{
    delete ui;
}

void startMain::on_actionIncrease_row_size_triggered()
{
    colSize++;
    init();
}

void startMain::on_actionDecrease_row_size_triggered()
{

    if(colSize>1)colSize--;
    init();
}
void startMain::on_adder_click()
{
    QDir *dir=new QDir("./Books");
    int num=dir->count()-1;
    char buff[100];
    QString name="Book "+QString(itoa(num,buff,10));
    qDebug()<<name<<" is getting created";
    if(dir->mkdir(name))
        init();
    else qDebug()<<name<<" is invalid file directory";
}


void startMain::on_book_click()
{
    //Retrive sender button's name
    QPushButton* buttonSender = qobject_cast<QPushButton*>(sender()); // retrieve the button you have clicked
    QString path = buttonSender->text();
    qDebug()<<path<<" clicked";
    BookViewer *book=new BookViewer();
    book->setWindowTitle(path.split("/")[path.split("/").length()-1]);
    qDebug()<<path<<" Sending";
    book->setPath(path);
    qDebug()<<path<<" Sent";

    book->setModal(true);
    book->init();
    book->showMaximized();
    book->show();
    book->showMaximized();
    book->exec();
    init();
}
