cmake_minimum_required(VERSION 3.12)

project(FreeSASA_test)

set(CMAKE_CXX_STANDARD 11)

message(STATUS "Include directories: ${CMAKE_INCLUDE_PATH}")

# Find the required libraries
find_library(FREESASA_LIB freesasa)
message(STATUS "FREESASA_LIB: ${FREESASA_LIB}")

if (NOT FREESASA_LIB)
    message(FATAL_ERROR "Could not find libfreesasa.a")
endif()

find_package(json-c CONFIG)

# LibXml2
find_package(LibXml2 REQUIRED)
include_directories(${LIBXML2_INCLUDE_DIR})
if (NOT LIBXML2_FOUND)
    message(FATAL_ERROR "Could not find libxml2")
endif()

# Add boost
find_package(Boost REQUIRED)
include_directories(${Boost_INCLUDE_DIRS})


# Add the libraries
add_library(xml2 STATIC IMPORTED)
set_target_properties(xml2 PROPERTIES IMPORTED_LOCATION ${LIBXML2_LIBRARIES})

# add_library(json-c STATIC IMPORTED)
# set_target_properties(json-c PROPERTIES IMPORTED_LOCATION ${LIBJSON_C})

add_library(freesasa STATIC IMPORTED)
set_target_properties(freesasa PROPERTIES IMPORTED_LOCATION ${FREESASA_LIB})

# Link the libraries
add_executable(FreeSASA_test main.cxx)
target_link_libraries(
        FreeSASA_test
        PRIVATE
        ${FREESASA_LIB}
        LibXml2::LibXml2
        json-c::json-c
        ${Boost_LIBRARIES}
)
