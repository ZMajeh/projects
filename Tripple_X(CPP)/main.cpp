#include <iostream>
#include "methods.h"

int main()
{
    int level=1,villainFactor;
    while (level<=10)
    {
        villainFactor=entryMSG();
        if(playAndCheckWin(level,villainFactor+1))
            winMsg();
        else
            loseMsg();
        level++;
    }
    std::cout << "Congratulations you just finished the whole game.\n Thanks for playing\nCredits: ZMajeh";
    return 0;
}