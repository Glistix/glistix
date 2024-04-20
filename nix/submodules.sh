# Links each submodule to its source in the Nix store.
# Workaround over Git submodules not being copied from source.
# See also https://github.com/NixOS/nix/issues/6633
IFS=$'\n'
for submodule in $submodules;
do
  subsrc="$(echo $submodule | cut -d':' -f1)"
  subdest="$(echo $submodule | cut -d':' -f2)"
  mkdir -p "$(dirname $subdest)"
  ln -s "$subsrc" -T "$subdest"
done
