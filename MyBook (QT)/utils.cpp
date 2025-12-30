#include "utils.h"

bool
bookUtils::naturalLessThan(const QFileInfo &s1, const QFileInfo &s2) {
    QString a = s1.fileName();
    QString b = s2.fileName();
    
    int i = 0, j = 0;
    while (i < a.length() && j < b.length()) {
        if (a[i].isDigit() && b[j].isDigit()) {
            // Extract full number from both strings
            QString numA, numB;
            while (i < a.length() && a[i].isDigit()) numA += a[i++];
            while (j < b.length() && b[j].isDigit()) numB += b[j++];
            
            // Compare numerically
            int valA = numA.toInt();
            int valB = numB.toInt();
            if (valA != valB) return valA < valB;
        } else {
            // Compare characters normally
            if (a[i].toLower() != b[j].toLower()) 
                return a[i].toLower() < b[j].toLower();
            i++; j++;
        }
    }
    return a.length() < b.length();
}