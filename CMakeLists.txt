cmake_minimum_required(VERSION 3.13.0)
project(vinylla VERSION 0.0.0)

#if(ENABLE_LTO)
#  set(RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang-11" "-Clink-arg=-fuse-ld=lld-11")
#endif()

set(RUST_CRATE_TYPE "staticlib")
set(RUST_LIBRARY_FILE "${CMAKE_STATIC_LIBRARY_PREFIX}vinylla${CMAKE_STATIC_LIBRARY_SUFFIX}")
if(BUILD_SHARED_LIBS)
  set(RUST_CRATE_TYPE "cdylib")
  set(RUST_LIBRARY_FILE "${CMAKE_SHARED_LIBRARY_PREFIX}vinylla${CMAKE_SHARED_LIBRARY_SUFFIX}")
endif()

add_custom_command(
  OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/${RUST_LIBRARY_FILE}" "${CMAKE_CURRENT_BINARY_DIR}/vinylla.h"
  COMMAND CARGO_GENERATE_C_HEADER=1 cargo rustc --lib $<$<CONFIG:Release>:--release> -- --crate-type=${RUST_CRATE_TYPE}
  COMMAND "${CMAKE_COMMAND}" -E copy_if_different "$<IF:$<CONFIG:Release>,target/release,target/debug>/${RUST_LIBRARY_FILE}" "${CMAKE_CURRENT_BINARY_DIR}"
  COMMAND "${CMAKE_COMMAND}" -E copy_if_different "target/vinylla.h" "${CMAKE_CURRENT_BINARY_DIR}"
  WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)
add_custom_target(vinylla ALL DEPENDS "${CMAKE_CURRENT_BINARY_DIR}/${RUST_LIBRARY_FILE}" "${CMAKE_CURRENT_BINARY_DIR}/vinylla.h")


add_test(
  NAME vinylla_test
  COMMAND cargo test
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

include(GNUInstallDirs)

install(
  FILES "${CMAKE_CURRENT_BINARY_DIR}/vinylla.h"
  DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/vinylla.h"
)
if(BUILD_SHARED_LIBS AND WIN32)
  install(
    FILES "${CMAKE_CURRENT_BINARY_DIR}/${RUST_LIBRARY_FILE}"
    DESTINATION "${CMAKE_INSTALL_BINDIR}/${RUST_LIBRARY_FILE}"
  )
else()
  install(
    FILES "${CMAKE_CURRENT_BINARY_DIR}/${RUST_LIBRARY_FILE}"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/${RUST_LIBRARY_FILE}"
  )
endif()
