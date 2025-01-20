#!/bin/sh
# Replaces 'gleam_core__' with 'glistix_core__' before all test snapshot (.snap) names
# Has some additional code to make it pretty because why not

ANSI_BOLD=$'\e[1m'
ANSI_BLUE=$'\e[34m'
ANSI_GREEN=$'\e[32m'
ANSI_YELLOW=$'\e[33m'
ANSI_END=$'\e[0m'

echo "${ANSI_BOLD}=== Replacing 'gleam_core__' with 'glistix_core__' in test snapshot names ===${ANSI_END}" >&2

curr_dir=""
files=$(find . -name 'gleam_core__*.snap' -o -name 'gleam__*.snap')
IFS=$'\n'
for i in ${files}
do
  # Replace the prefix
  newname="$(echo "$i" | sed -Ee 's;(^|/)gleam_core__;\1glistix_core__;' | sed -Ee 's;(^|/)gleam__;\1glistix__;')"

  # --- Display info ---
  dir="$(dirname "$i")"
  if [ "$curr_dir" != "$dir" ]; then
    echo "" >&2
    echo "${ANSI_BLUE}*${ANSI_END} Directory ${ANSI_YELLOW}${ANSI_BOLD}${dir}${ANSI_END}:" >&2
    curr_dir="$dir"
  fi
  echo "	${ANSI_BLUE}*${ANSI_END} Renaming ${ANSI_GREEN}${ANSI_BOLD}$(basename "${i}")${ANSI_END} -> ${ANSI_GREEN}${ANSI_BOLD}$(basename "${newname}")${ANSI_END}." >&2
  # --------------------

  # Move it!
  mv "$i" -T "$newname" 
done

# --- Display info ---
echo "" >&2
echo "${ANSI_BOLD}=== Renamed $(printf "${files}" | wc -l) files ===${ANSI_END}" >&2
