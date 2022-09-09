cmake_minimum_required(VERSION 3.19)
project(ramu_rs)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
set(CMAKE_CXX_STANDARD 20)

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.2.1 # Optionally specify a commit hash, version tag or branch here
)

FetchContent_MakeAvailable(Corrosion)
corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)

add_library(ramu_rs_cpp ${CMAKE_CURRENT_SOURCE_DIR}/ramu_rs_cpp.cc)
target_link_libraries(ramu_rs_cpp PUBLIC ramu_rs)
target_include_directories(ramu_rs_cpp PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

add_executable(ramu_rs_cpp_test ${CMAKE_CURRENT_SOURCE_DIR}/test.cc)
target_link_libraries(ramu_rs_cpp_test PUBLIC ramu_rs_cpp)