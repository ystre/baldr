cmake_minimum_required(VERSION 3.5)

project("Test")

if(SANITIZERS)
    if(SANITIZERS STREQUAL asan)
        set(SANITIZER_LIST -fsanitize=address -fsanitize=leak -fsanitize=undefined)
    endif()
    if(SANITIZERS STREQUAL tsan)
        set(SANITIZER_LIST -fsanitize=thread)
    endif()
endif()

add_executable(test main.cc)
target_compile_options(test PRIVATE ${SANITIZER_LIST})
target_link_options(test PRIVATE ${SANITIZER_LIST})

if(DEFINE1)
    target_compile_definitions(test PRIVATE DEFINE1=${DEFINE1})
endif()

if(DEFINE2)
    target_compile_definitions(test PRIVATE DEFINE2=${DEFINE2})
endif()

if(CONFIGURE_FAILURE)
    find_package(non-existent REQUIRED)
endif()
