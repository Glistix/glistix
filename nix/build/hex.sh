# Populates Gleam's cache of hex packages.
case $(uname | tr '[:upper:]' '[:lower:]') in
  darwin*)
    # MacOS has a different cache location
    export HOME="$(mktemp -d)/.gleam_cache"
    hexdir="${HOME}/Library/Cache/gleam/hex/hexpm/packages"
    ;;
  *)
    # Assume Linux
    export XDG_CACHE_HOME="$(mktemp -d)/.gleam_cache"
    hexdir="${XDG_CACHE_HOME}/gleam/hex/hexpm/packages"
    ;;
esac

mkdir -p "$hexdir"
IFS=$'\n'
for package in $fetchedHexPackagePaths;
do
  dest="$hexdir/$(basename "$package" | cut -d '-' -f1 --complement)"
  echo "Linking $package to $dest"
  ln -s "$package" -T "$dest"
done
