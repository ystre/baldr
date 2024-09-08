#include <cstdlib>
#include <iostream>

#define STRINGIFY(x) #x
#define TOSTRING(x) STRINGIFY(x)

int main(int argc, char* args[]) {
    std::cout << "Arguments: ";

    for (int i = 1; i < argc; ++i) {
        std::cout << args[i] << ' ';
    }

    std::cout << '\n';

    std::cout << "Defines: ";
    #ifdef DEFINE1
    std::cout << TOSTRING(DEFINE1) << ' ';
    #endif

    #ifdef DEFINE2
    std::cout << TOSTRING(DEFINE2) << ' ';
    #endif

    std::cout << '\n';

    return EXIT_SUCCESS;
}
