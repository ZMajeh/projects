#ifndef BOOKUtils_H
#define BOOKUtils_H

#include<QMainWindow>
#include<QDir>
#include<QFile>
#include<QVBoxLayout>
#include<QtCore>
#include<QPushButton>
#include<QWidget>
#include<QGraphicsPixmapItem>
#include<QGraphicsView>
#include<QDebug>
#include<algorithm>
#include<QDialog>
#include<QDir>
#include<QFile>
#include<QVBoxLayout>
#include<QtCore>
#include<QPushButton>
#include<QWidget>
#include<QGraphicsPixmapItem>
#include<QGraphicsView>
#include<QDebug>
#include<QLabel>

#include<startmain.h>
#include<bookviewer.h>

class bookUtils
{
    public:
        // Natural comparison function for Qt 4.8
        static bool naturalLessThan(const QFileInfo &s1, const QFileInfo &s2);
};

#endif // BOOKUtils_H