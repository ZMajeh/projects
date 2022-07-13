#ifndef METHODS
#define METHODS
#include <iostream>
#include <cstdlib>
#include <time.h>

std::string villains[]={
    "Luffy, this will be easy",
    "Zoro, now you will have hard time",
    "Mega Mind, you are gonner dude"
};

int entryMSG()
{
    static int universe;
    int difficulty;
    srand(time(0));
    difficulty=rand()%3;
    if(universe==0)
        universe=rand()%1000;
    else universe++;
    std::cout << "\n\n\n=============================================================\nWelcome to universe : " << universe <<
                 "\nAs soon you enter this universe you are faced with huge barricade\n" <<
                 "You see there is an input pad and it is asking for password\n" <<
                 "This device is developed by " << villains[difficulty] << std::endl;
    return difficulty;
}

bool playAndCheckWin(int difficulty, int villain)
{
    const int num1=(rand()%difficulty+difficulty) * villain * villain,
              num2=(rand()%difficulty+difficulty) * villain * villain,
              num3=(rand()%difficulty+difficulty) * villain * villain;
    const int sum=num1+num2+num3,
              product=num1*num2*num3;
    int guessNum1,guessNum2,guessNum3,guessSum,guessProduct;

    std::cout << "Small digital screen and shows you following : \n" <<
                 "***********************************************\n" <<
                 "***********************************************\n" <<
                 "****          Welcome human                ****\n" <<
                 "****     This will result in your death    ****\n" <<
                 "**** Try solving this or bomb will trigger ****\n" <<
                 "****  3 numbers match following conditions ****\n" <<
                 "***********************************************\n" <<
                 "***********************************************\n" <<
                 "\nSum of them is : " << sum <<
                 "\nProduct of them is : " << product << std::endl <<
                 "\nYour input : "
                 ;
    std::cin >> guessNum1 >> guessNum2 >> guessNum3;
    guessSum = guessNum1 + guessNum2 + guessNum3;
    guessProduct = guessNum1 * guessNum2 * guessNum3;

    std::cout << "You entered : " << guessNum1 << " : " << guessNum2 << " : " << guessNum3 << std::endl;

    if( sum == guessSum && product == guessProduct )
        return true;
    else 
        return false;
}

void winMsg()
{
    std::cout << "Congratulations, you opened the gate\nNow you can head to portal of next universe";
}

void loseMsg()
{
    std::cout << "You failed and bomb is triggered, You see crow eating your eyeball and dogs biting on your guts...";
    exit(1);
}
#endif