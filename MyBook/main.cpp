#include <QtGui/QApplication>
#include "startmain.h"

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);
    startMain w;
    w.show();
    w.showMaximized();

    return a.exec();
}
