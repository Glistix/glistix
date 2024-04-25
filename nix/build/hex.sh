# Populates Gleam's cache of hex packages.
export XDG_CACHE_HOME="$(mktemp -d)/.gleam_cache"
hexdir="${XDG_CACHE_HOME}/gleam/hex/hexpm/packages"
mkdir -p "$hexdir"

IFS=$'\n'
for package in $fetchedHexPackagePaths;
do
  dest="$hexdir/$(basename "$package" | cut -d '-' -f1 --complement)"
  echo "Linking $package to $dest"
  ln -s "$package" -T "$dest"
done
