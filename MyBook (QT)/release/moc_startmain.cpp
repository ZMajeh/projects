/****************************************************************************
** Meta object code from reading C++ file 'startmain.h'
**
** Created by: The Qt Meta Object Compiler version 63 (Qt 4.8.7)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "../startmain.h"
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'startmain.h' doesn't include <QObject>."
#elif Q_MOC_OUTPUT_REVISION != 63
#error "This file was generated using the moc from 4.8.7. It"
#error "cannot be used with the include files from this version of Qt."
#error "(The moc has changed too much.)"
#endif

QT_BEGIN_MOC_NAMESPACE
static const uint qt_meta_data_ExportWorker[] = {

 // content:
       6,       // revision
       0,       // classname
       0,    0, // classinfo
       3,   14, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       3,       // signalCount

 // signals: signature, parameters, type, tag, flags
      20,   14,   13,   13, 0x05,
      42,   34,   13,   13, 0x05,
      68,   60,   13,   13, 0x05,

       0        // eod
};

static const char qt_meta_stringdata_ExportWorker[] = {
    "ExportWorker\0\0value\0progress(int)\0"
    "pdfPath\0finished(QString)\0message\0"
    "error(QString)\0"
};

void ExportWorker::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    if (_c == QMetaObject::InvokeMetaMethod) {
        Q_ASSERT(staticMetaObject.cast(_o));
        ExportWorker *_t = static_cast<ExportWorker *>(_o);
        switch (_id) {
        case 0: _t->progress((*reinterpret_cast< int(*)>(_a[1]))); break;
        case 1: _t->finished((*reinterpret_cast< QString(*)>(_a[1]))); break;
        case 2: _t->error((*reinterpret_cast< QString(*)>(_a[1]))); break;
        default: ;
        }
    }
}

const QMetaObjectExtraData ExportWorker::staticMetaObjectExtraData = {
    0,  qt_static_metacall 
};

const QMetaObject ExportWorker::staticMetaObject = {
    { &QThread::staticMetaObject, qt_meta_stringdata_ExportWorker,
      qt_meta_data_ExportWorker, &staticMetaObjectExtraData }
};

#ifdef Q_NO_DATA_RELOCATION
const QMetaObject &ExportWorker::getStaticMetaObject() { return staticMetaObject; }
#endif //Q_NO_DATA_RELOCATION

const QMetaObject *ExportWorker::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->metaObject : &staticMetaObject;
}

void *ExportWorker::qt_metacast(const char *_clname)
{
    if (!_clname) return 0;
    if (!strcmp(_clname, qt_meta_stringdata_ExportWorker))
        return static_cast<void*>(const_cast< ExportWorker*>(this));
    return QThread::qt_metacast(_clname);
}

int ExportWorker::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = QThread::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        if (_id < 3)
            qt_static_metacall(this, _c, _id, _a);
        _id -= 3;
    }
    return _id;
}

// SIGNAL 0
void ExportWorker::progress(int _t1)
{
    void *_a[] = { 0, const_cast<void*>(reinterpret_cast<const void*>(&_t1)) };
    QMetaObject::activate(this, &staticMetaObject, 0, _a);
}

// SIGNAL 1
void ExportWorker::finished(QString _t1)
{
    void *_a[] = { 0, const_cast<void*>(reinterpret_cast<const void*>(&_t1)) };
    QMetaObject::activate(this, &staticMetaObject, 1, _a);
}

// SIGNAL 2
void ExportWorker::error(QString _t1)
{
    void *_a[] = { 0, const_cast<void*>(reinterpret_cast<const void*>(&_t1)) };
    QMetaObject::activate(this, &staticMetaObject, 2, _a);
}
static const uint qt_meta_data_startMain[] = {

 // content:
       6,       // revision
       0,       // classname
       0,    0, // classinfo
       8,   14, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       0,       // signalCount

 // slots: signature, parameters, type, tag, flags
      11,   10,   10,   10, 0x08,
      50,   10,   10,   10, 0x08,
      89,   10,   10,   10, 0x0a,
     104,   10,   10,   10, 0x0a,
     118,   10,   10,   10, 0x08,
     147,  141,   10,   10, 0x08,
     177,  169,   10,   10, 0x08,
     211,  203,   10,   10, 0x08,

       0        // eod
};

static const char qt_meta_stringdata_startMain[] = {
    "startMain\0\0on_actionDecrease_row_size_triggered()\0"
    "on_actionIncrease_row_size_triggered()\0"
    "adderClicked()\0bookClicked()\0"
    "onDownloadBtnClicked()\0value\0"
    "onExportProgress(int)\0pdfPath\0"
    "onExportFinished(QString)\0message\0"
    "onExportError(QString)\0"
};

void startMain::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    if (_c == QMetaObject::InvokeMetaMethod) {
        Q_ASSERT(staticMetaObject.cast(_o));
        startMain *_t = static_cast<startMain *>(_o);
        switch (_id) {
        case 0: _t->on_actionDecrease_row_size_triggered(); break;
        case 1: _t->on_actionIncrease_row_size_triggered(); break;
        case 2: _t->adderClicked(); break;
        case 3: _t->bookClicked(); break;
        case 4: _t->onDownloadBtnClicked(); break;
        case 5: _t->onExportProgress((*reinterpret_cast< int(*)>(_a[1]))); break;
        case 6: _t->onExportFinished((*reinterpret_cast< QString(*)>(_a[1]))); break;
        case 7: _t->onExportError((*reinterpret_cast< QString(*)>(_a[1]))); break;
        default: ;
        }
    }
}

const QMetaObjectExtraData startMain::staticMetaObjectExtraData = {
    0,  qt_static_metacall 
};

const QMetaObject startMain::staticMetaObject = {
    { &QMainWindow::staticMetaObject, qt_meta_stringdata_startMain,
      qt_meta_data_startMain, &staticMetaObjectExtraData }
};

#ifdef Q_NO_DATA_RELOCATION
const QMetaObject &startMain::getStaticMetaObject() { return staticMetaObject; }
#endif //Q_NO_DATA_RELOCATION

const QMetaObject *startMain::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->metaObject : &staticMetaObject;
}

void *startMain::qt_metacast(const char *_clname)
{
    if (!_clname) return 0;
    if (!strcmp(_clname, qt_meta_stringdata_startMain))
        return static_cast<void*>(const_cast< startMain*>(this));
    return QMainWindow::qt_metacast(_clname);
}

int startMain::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = QMainWindow::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        if (_id < 8)
            qt_static_metacall(this, _c, _id, _a);
        _id -= 8;
    }
    return _id;
}
QT_END_MOC_NAMESPACE
