/****************************************************************************
** Meta object code from reading C++ file 'startmain.h'
**
** Created: Sun 24. Apr 17:22:14 2022
**      by: The Qt Meta Object Compiler version 62 (Qt 4.7.0)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "../../MyBook/startmain.h"
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'startmain.h' doesn't include <QObject>."
#elif Q_MOC_OUTPUT_REVISION != 62
#error "This file was generated using the moc from 4.7.0. It"
#error "cannot be used with the include files from this version of Qt."
#error "(The moc has changed too much.)"
#endif

QT_BEGIN_MOC_NAMESPACE
static const uint qt_meta_data_startMain[] = {

 // content:
       5,       // revision
       0,       // classname
       0,    0, // classinfo
       4,   14, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       0,       // signalCount

 // slots: signature, parameters, type, tag, flags
      11,   10,   10,   10, 0x08,
      50,   10,   10,   10, 0x08,
      89,   10,   10,   10, 0x0a,
     106,   10,   10,   10, 0x0a,

       0        // eod
};

static const char qt_meta_stringdata_startMain[] = {
    "startMain\0\0on_actionDecrease_row_size_triggered()\0"
    "on_actionIncrease_row_size_triggered()\0"
    "on_adder_click()\0on_book_click()\0"
};

const QMetaObject startMain::staticMetaObject = {
    { &QMainWindow::staticMetaObject, qt_meta_stringdata_startMain,
      qt_meta_data_startMain, 0 }
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
        switch (_id) {
        case 0: on_actionDecrease_row_size_triggered(); break;
        case 1: on_actionIncrease_row_size_triggered(); break;
        case 2: on_adder_click(); break;
        case 3: on_book_click(); break;
        default: ;
        }
        _id -= 4;
    }
    return _id;
}
QT_END_MOC_NAMESPACE
