/****************************************************************************
** Meta object code from reading C++ file 'bookviewer.h'
**
** Created: Sun 24. Apr 17:22:15 2022
**      by: The Qt Meta Object Compiler version 62 (Qt 4.7.0)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "../../MyBook/bookviewer.h"
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'bookviewer.h' doesn't include <QObject>."
#elif Q_MOC_OUTPUT_REVISION != 62
#error "This file was generated using the moc from 4.7.0. It"
#error "cannot be used with the include files from this version of Qt."
#error "(The moc has changed too much.)"
#endif

QT_BEGIN_MOC_NAMESPACE
static const uint qt_meta_data_BookViewer[] = {

 // content:
       5,       // revision
       0,       // classname
       0,    0, // classinfo
       3,   14, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       0,       // signalCount

 // slots: signature, parameters, type, tag, flags
      12,   11,   11,   11, 0x0a,
      29,   11,   11,   11, 0x0a,
      48,   11,   11,   11, 0x0a,

       0        // eod
};

static const char qt_meta_stringdata_BookViewer[] = {
    "BookViewer\0\0on_adder_click()\0"
    "on_remover_click()\0on_page_click()\0"
};

const QMetaObject BookViewer::staticMetaObject = {
    { &QDialog::staticMetaObject, qt_meta_stringdata_BookViewer,
      qt_meta_data_BookViewer, 0 }
};

#ifdef Q_NO_DATA_RELOCATION
const QMetaObject &BookViewer::getStaticMetaObject() { return staticMetaObject; }
#endif //Q_NO_DATA_RELOCATION

const QMetaObject *BookViewer::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->metaObject : &staticMetaObject;
}

void *BookViewer::qt_metacast(const char *_clname)
{
    if (!_clname) return 0;
    if (!strcmp(_clname, qt_meta_stringdata_BookViewer))
        return static_cast<void*>(const_cast< BookViewer*>(this));
    return QDialog::qt_metacast(_clname);
}

int BookViewer::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = QDialog::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        switch (_id) {
        case 0: on_adder_click(); break;
        case 1: on_remover_click(); break;
        case 2: on_page_click(); break;
        default: ;
        }
        _id -= 3;
    }
    return _id;
}
QT_END_MOC_NAMESPACE
