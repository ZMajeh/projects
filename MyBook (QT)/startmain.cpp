#include "startmain.h"
#include "ui_startmain.h"

#include <QFileDialog>
// #include <QPdfWriter>
#include <QPainter>
#include <QPrinter>
#include <QDir>
#include <QImage>
// #include <QCollator>
#include <QMessageBox>
#include <algorithm>
#include <QToolButton>
#include <QStyle>
#include <QProgressDialog>


#include "startMain.h"

// ---------------- Worker Implementation ----------------
ExportWorker::ExportWorker(QString dirPath, QString savePath, QObject *parent)
    : QThread(parent), m_dirPath(dirPath), m_savePath(savePath) {}

void ExportWorker::run() {
    QDir dir(m_dirPath);
    QStringList nameFilters;
    nameFilters << "*.png" << "*.PNG";
    QFileInfoList fileList = dir.entryInfoList(nameFilters, QDir::Files | QDir::NoSymLinks);

    if (fileList.isEmpty()) {
        emit error(QObject::tr("No PNG images found in the selected folder."));
        return;
    }
    
    qSort(fileList.begin(), fileList.end(), bookUtils::naturalLessThan);

    // Ensure template.png is first
    for (int i = 0; i < fileList.size(); ++i) {
        if (fileList.at(i).fileName().toLower() == "template.png") {
            QFileInfo t = fileList.takeAt(i);
            fileList.insert(0, t);
            break;
        }
    }

    QPrinter printer(QPrinter::HighResolution);
    // Use lower DPI for smaller embedded images (72–150). Adjust to taste.
    printer.setResolution(96);
    printer.setOutputFormat(QPrinter::PdfFormat);
    printer.setFullPage(true);
    // printer.setPaperSize(QPrinter::A4);
    printer.setOutputFileName(m_savePath);

    QPainter painter(&printer);
    if (!painter.isActive()) {
        emit error(QObject::tr("Failed to create PDF writer."));
        return;
    }

    int lastIndex = fileList.size() - 1;
    for (int i = 0; i <= lastIndex; ++i) {
        emit progress((i * 100) / (lastIndex > 0 ? lastIndex : 1));

        const QFileInfo &fi = fileList.at(i);
        QImage img(fi.absoluteFilePath());
        if (img.isNull()) {
            if (i != lastIndex) printer.newPage();
            continue;
        }

        QSize pageSize(printer.pageRect().size());
        QImage scaled = img.scaled(pageSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);

        // Compress to JPEG before painting
        QByteArray ba;
        QBuffer buffer(&ba);
        buffer.open(QIODevice::WriteOnly);
        // reduce quality to 60 to shrink size further
        // scaled.save(&buffer, "JPEG", 60);
        scaled.save(&buffer, "PNG", 100);
        QImage compressed = QImage::fromData(ba, "PNG");

        int x = (pageSize.width() - compressed.width()) / 2;
        int y = (pageSize.height() - compressed.height()) / 2;
        painter.save();
        painter.drawImage(QPoint(x, y), compressed);

        // Draw filename centered on bottom with semi-transparent background
        QString name = fi.fileName();
        QFont f = painter.font();
        f.setPointSize(12); // adjust size as needed
        painter.setFont(f);
        QFontMetrics fm(f);
        int textW = fm.width(name); // Qt4 API
        int textH = fm.height();
        int margin = 6;
        QRect bgRect((pageSize.width() - textW) / 2 - margin,
                     pageSize.height() - textH - margin*2,
                     textW + margin*2,
                     textH + margin*2);

        painter.setPen(Qt::NoPen);
        painter.setBrush(QColor(0,0,0,150)); // semi-transparent black
        painter.drawRect(bgRect);

        painter.setPen(Qt::white);
        painter.drawText(bgRect, Qt::AlignCenter, name);
        painter.restore();

        if (i != lastIndex) printer.newPage();
    }

    painter.end();
    emit progress(100);
    emit finished(m_savePath);
}

void startMain::onDownloadBtnClicked() {
    QToolButton* buttonSender = qobject_cast<QToolButton*>(sender());
    QString dirPath = buttonSender->property("path").toString();
    if (dirPath.isEmpty()) return;
    qDebug() << "Exporting directory:" << dirPath;

    QString savePath = QFileDialog::getSaveFileName(this, tr("Save PDF"),
                        dirPath + "/output.pdf", tr("PDF Files (*.pdf)"));
    if (savePath.isEmpty()) return;

    progressDialog = new QProgressDialog(tr("Exporting images to PDF..."), tr("Cancel"), 0, 100, this);
    progressDialog->setWindowModality(Qt::ApplicationModal);
    progressDialog->setMinimumDuration(0);

    ExportWorker *worker = new ExportWorker(dirPath, savePath, this);
    connect(worker, SIGNAL(progress(int)), this, SLOT(onExportProgress(int)));
    connect(worker, SIGNAL(finished(QString)), this, SLOT(onExportFinished(QString)));
    connect(worker, SIGNAL(error(QString)), this, SLOT(onExportError(QString)));
    worker->start();
}

void startMain::onExportProgress(int value) {
    if (progressDialog) progressDialog->setValue(value);
}

void startMain::onExportFinished(QString pdfPath) {
    if (progressDialog) progressDialog->close();
    QMessageBox::information(this, tr("Done"), tr("PDF saved to:\n%1").arg(pdfPath));
}

void startMain::onExportError(QString message) {
    if (progressDialog) progressDialog->close();
    QMessageBox::critical(this, tr("Error"), message);
}



// void startMain::exportDirToPdf(QString dirPath)
// {
//     if (dirPath.isEmpty()) return;

//     QDir dir(dirPath);
//     QStringList nameFilters;
//     nameFilters << "*.png" << "*.PNG";
//     QFileInfoList fileList = dir.entryInfoList(nameFilters, QDir::Files | QDir::NoSymLinks);

//     if (fileList.isEmpty()) {
//         QMessageBox::warning(this, tr("No images"), tr("No PNG images found in the selected folder."));
//         return;
//     }

//     // Sort filenames (Qt4 style)
//     qSort(fileList.begin(), fileList.end(), bookUtils::naturalLessThan);

//     // Ensure template.png is first
//     for (int i = 0; i < fileList.size(); ++i) {
//         if (fileList.at(i).fileName().toLower() == "template.png") {
//             QFileInfo t = fileList.takeAt(i);
//             fileList.insert(0, t);
//             break;
//         }
//     }

//     QString savePath = QFileDialog::getSaveFileName(this, tr("Save PDF"),
//                         dirPath + "/output.pdf", tr("PDF Files (*.pdf)"));
//     if (savePath.isEmpty()) return;

//     QPrinter printer(QPrinter::HighResolution);
//     printer.setOutputFormat(QPrinter::PdfFormat);
//     printer.setPaperSize(QPrinter::A4);
//     printer.setOutputFileName(savePath);

//     QPainter painter(&printer);
//     if (!painter.isActive()) {
//         QMessageBox::critical(this, tr("Error"), tr("Failed to create PDF writer."));
//         return;
//     }

//     QProgressDialog progress(tr("Exporting images to PDF..."), tr("Cancel"), 0, fileList.size(), this);
//     progress.setWindowModality(Qt::ApplicationModal);
//     progress.setMinimumDuration(0);

//     int lastIndex = fileList.size() - 1;
//     for (int i = 0; i <= lastIndex; ++i) {
//         progress.setValue(i);
//         if (progress.wasCanceled()) {
//             painter.end();
//             return;
//         }

//         const QFileInfo &fi = fileList.at(i);
//         QImage img(fi.absoluteFilePath());
//         if (img.isNull()) {
//             if (i != lastIndex) printer.newPage();
//             continue;
//         }

//         QSize pageSize(printer.pageRect().size());
//         QImage scaled = img.scaled(pageSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);

//         // Compress to JPEG before painting
//         QByteArray ba;
//         QBuffer buffer(&ba);
//         buffer.open(QIODevice::WriteOnly);
//         scaled.save(&buffer, "JPEG", 75); // quality 0-100
//         QImage compressed = QImage::fromData(ba, "JPEG");

//         int x = (pageSize.width() - compressed.width()) / 2;
//         int y = (pageSize.height() - compressed.height()) / 2;
//         painter.drawImage(QPoint(x, y), compressed);

//         if (i != lastIndex) printer.newPage();
//     }

//     painter.end();
//     progress.setValue(fileList.size());

//     QMessageBox::information(this, tr("Done"), tr("PDF saved to:\n%1").arg(savePath));
// }


// void startMain::exportDirToPdf(QString dirPath)
// {
//     if (dirPath.isEmpty()) return;

//     QDir dir(dirPath);
//     QStringList nameFilters;
//     nameFilters << "*.png" << "*.PNG";
//     QFileInfoList fileList = dir.entryInfoList(nameFilters, QDir::Files | QDir::NoSymLinks);

//     if (fileList.isEmpty()) {
//         QMessageBox::warning(this, tr("No images"), tr("No PNG images found in the selected folder."));
//         return;
//     }

//     // Sort filenames (locale-aware, but not numeric-aware)
//     qSort(fileList.begin(), fileList.end(), 
//         [](const QFileInfo &a, const QFileInfo &b) {
//             return QString::localeAwareCompare(a.fileName(), b.fileName()) < 0;
//         }
//     );
//     // ⚠️ In Qt4 you cannot use lambdas. Replace with a static function:
//     // static bool fileInfoLess(const QFileInfo &a, const QFileInfo &b) {
//     //     return QString::localeAwareCompare(a.fileName(), b.fileName()) < 0;
//     // }
//     // qSort(fileList.begin(), fileList.end(), fileInfoLess);

//     // Ensure template.png is first
//     for (int i = 0; i < fileList.size(); ++i) {
//         if (fileList.at(i).fileName().toLower() == "template.png") {
//             QFileInfo t = fileList.takeAt(i);
//             fileList.insert(0, t);
//             break;
//         }
//     }

//     QString savePath = QFileDialog::getSaveFileName(this, tr("Save PDF"),
//                         dirPath + "/output.pdf", tr("PDF Files (*.pdf)"));
//     if (savePath.isEmpty()) return;

//     QPrinter printer(QPrinter::HighResolution);
//     printer.setOutputFormat(QPrinter::PdfFormat);
//     printer.setPaperSize(QPrinter::A4);
//     printer.setOutputFileName(savePath);

//     QPainter painter(&printer);
//     if (!painter.isActive()) {
//         QMessageBox::critical(this, tr("Error"), tr("Failed to create PDF writer."));
//         return;
//     }

//     int lastIndex = fileList.size() - 1;
//     for (int i = 0; i <= lastIndex; ++i) {
//         const QFileInfo &fi = fileList.at(i);
//         QImage img(fi.absoluteFilePath());
//         if (img.isNull()) {
//             if (i != lastIndex) printer.newPage();
//             continue;
//         }

//         QSize pageSize(printer.pageRect().size());
//         QImage scaled = img.scaled(pageSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
//         int x = (pageSize.width() - scaled.width()) / 2;
//         int y = (pageSize.height() - scaled.height()) / 2;
//         painter.drawImage(QPoint(x, y), scaled);

//         if (i != lastIndex) printer.newPage();
//     }

//     painter.end();
//     QMessageBox::information(this, tr("Done"), tr("PDF saved to:\n%1").arg(savePath));
// }


// void startMain::exportDirToPdf(QString dirPath)
// {
//     if (dirPath.isEmpty()) return;

//     QDir dir(dirPath);
//     QStringList nameFilters;
//     nameFilters << "*.png" << "*.PNG";
//     QFileInfoList fileList = dir.entryInfoList(nameFilters, QDir::Files | QDir::NoSymLinks);

//     if (fileList.isEmpty()) {
//         QMessageBox::warning(this, tr("No images"), tr("No PNG images found in the selected folder."));
//         return;
//     }

//     // Natural / numeric-aware sort by filename
//     QCollator coll;
//     coll.setNumericMode(true);
//     std::sort(fileList.begin(), fileList.end(), [&](const QFileInfo &a, const QFileInfo &b){
//         return coll.compare(a.fileName(), b.fileName()) < 0;
//     });

//     // Ensure template.png (case-insensitive) is first if present
//     for (int i = 0; i < fileList.size(); ++i) {
//         if (fileList.at(i).fileName().toLower() == "template.png") {
//             QFileInfo t = fileList.takeAt(i);
//             fileList.insert(0, t);
//             break;
//         }
//     }

//     QString savePath = QFileDialog::getSaveFileName(this, tr("Save PDF"), dirPath + "/output.pdf", tr("PDF Files (*.pdf)"));
//     if (savePath.isEmpty()) return;

//     QPdfWriter writer(savePath);
//     writer.setPageSize(QPagedPaintDevice::A4);
//     writer.setResolution(300);

//     QPainter painter(&writer);
//     if (!painter.isActive()) {
//         QMessageBox::critical(this, tr("Error"), tr("Failed to create PDF writer."));
//         return;
//     }

//     int lastIndex = fileList.size() - 1;
//     for (int i = 0; i <= lastIndex; ++i) {
//         const QFileInfo &fi = fileList.at(i);
//         QImage img(fi.absoluteFilePath());
//         if (img.isNull()) {
//             // skip invalid images
//             if (i != lastIndex) writer.newPage();
//             continue;
//         }

//         QSize pageSize(writer.width(), writer.height());
//         QImage scaled = img.scaled(pageSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
//         int x = (pageSize.width() - scaled.width()) / 2;
//         int y = (pageSize.height() - scaled.height()) / 2;
//         painter.drawImage(QPoint(x, y), scaled);

//         if (i != lastIndex) writer.newPage();
//     }

//     painter.end();
//     QMessageBox::information(this, tr("Done"), tr("PDF saved to:\n%1").arg(savePath));
// }

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
        connect(b1,SIGNAL(clicked()),this,SLOT(bookClicked()));
    out1->addWidget(b1);
    out->setLayout(out1);

    QLabel *infoLabel = new QLabel();
    //out1->addStretch();
    QDir bookDir(path);
    bookDir.setFilter(QDir::Files);
    int pages = bookDir.count();
    QFileInfo dirInfo(path);
    QString sizeStr;
    qint64 totalSize = 0;
    foreach(QString file, bookDir.entryList()) {
        totalSize += QFileInfo(bookDir.filePath(file)).size();
    }
    if(totalSize > 1024*1024) {
        sizeStr = QString::number(totalSize / (1024.0*1024.0), 'f', 1) + " MB";
    } else {
        sizeStr = QString::number(totalSize / 1024.0, 'f', 1) + " KB";
    }
    infoLabel->setText(QString("Pages: %1 | Size: %2").arg(pages).arg(sizeStr));
    infoLabel->setAlignment(Qt::AlignCenter);
    infoLabel->setStyleSheet("color: white; font-weight: bold; background-color: rgba(0,0,0,150);");

    // Ensure only the first widget expands vertically
    b1->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    infoLabel->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Fixed);

    QHBoxLayout *hLayout = new QHBoxLayout();
    infoLabel->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);

    QToolButton *downloadBtn = new QToolButton();
    QIcon dlIcon = QIcon::fromTheme("download");
    if (dlIcon.isNull()) dlIcon = style()->standardIcon(QStyle::SP_DialogSaveButton);
    downloadBtn->setIcon(dlIcon);
    downloadBtn->setToolTip(tr("Download"));
    downloadBtn->setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
    downloadBtn->setMaximumSize(24, 24);

    hLayout->addWidget(infoLabel);
    hLayout->addWidget(downloadBtn);

    out1->addLayout(hLayout);
    // Assign the path string to the button as a dynamic property 
    downloadBtn->setProperty("path", QVariant(path));
    connect(downloadBtn, SIGNAL(clicked()), this, SLOT(onDownloadBtnClicked()));

    infoLabel->setMaximumHeight(infoLabel->sizeHint().height());
    out1->setStretch(0, 1);
    out1->setStretch(1, 0);

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
        // Set up the "Natural Sort" collator
        QFileInfoList fileList = dir->entryInfoList();

        // Perform the sort
        // Use qSort (standard in Qt 4) with the custom helper
        qSort(fileList.begin(), fileList.end(), bookUtils::naturalLessThan);

        foreach(QFileInfo file, fileList)
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
        connect(b1,SIGNAL(clicked()),this,SLOT(adderClicked()));
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

void startMain::adderClicked()
{
    QDir *dir=new QDir("./Books");
    int num=dir->count()-1;
    char buff[100];
    QString name="Book "+QString(itoa(num,buff,10));
    qDebug()<<name<<" is getting created";

    // Find unique folder name
    bool created = false;
    int tries = 0;
    while(!created && tries < 10000) {
        if(dir->mkdir(name)) {
            created = true;
        } else {
            num++;
            tries++;
            name = "Book " + QString(itoa(num, buff, 10));
        }
    }

    // Verify creation
    if(dir->exists(name))
        init();
    else qDebug()<<name<<" is invalid file directory";
}


void startMain::bookClicked()
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
