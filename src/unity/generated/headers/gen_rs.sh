#!/bin/bash
set -e

bindgenArgs=(
  "--no-layout-tests"
  "--impl-debug"
  "--with-derive-default"
  "--no-doc-comments"
  "--enable-cxx-namespaces"
  "--disable-header-comment"
  "--ignore-functions"
  "--no-prepend-enum-name"
  "--use-array-pointers-in-arguments"
  "--raw-line", "#![allow(unused_qualifications)]",
  "--raw-line", "#![allow(unsafe_op_in_unsafe_fn)]"
  "--raw-line", "#![allow(clippy::useless_transmute)]"
  "--raw-line", "#![allow(clippy::too_many_arguments)]"
  "--raw-line", "#![allow(clippy::ptr_offset_with_cast)]"
)

for header in *.h; do
  headerFileFullPath=$(realpath "$header")
  headerFileName=$(basename "$header")
  headerDirectory=$(dirname "$headerFileFullPath")

  baseName=$(basename "$headerFileName" .h)
  baseName=$(echo "$baseName" | sed 's/[^a-zA-Z0-9]//g')

  outputFile="${headerDirectory}/../il2cpp_${baseName}.rs"

  echo "Processing '$headerFileName' -> '$outputFile'"

  currentBindgenArgs=( "$headerFileFullPath" "-o" "$outputFile" )
  currentBindgenArgs+=( "${bindgenArgs[@]}" )

  clangArgs=( "--target=aarch64-linux-android" "-x" "c++" )

  finalCmdArgs=( "${currentBindgenArgs[@]}" "--" "${clangArgs[@]}" )

  echo "  Command: bindgen.exe ${finalCmdArgs[*]}"

  if ! bindgen.exe "${finalCmdArgs[@]}"; then
      echo "  bindgen.exe failed for '$headerFileName'" >&2
  else
      echo "  Successfully generated '$outputFile'"
  fi

  echo ""
done

echo "Script finished."