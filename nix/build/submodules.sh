# Links each submodule to its source in the Nix store.
# Workaround over Git submodules not being copied from source.
# See also https://github.com/NixOS/nix/issues/6633
IFS=$'\n'
for submodule in $submodules;
do
  subsrc="$(echo "$submodule" | cut -d':' -f1)"
  subdest="$(echo "$submodule" | cut -d':' -f2)"

  # Remove empty directory in case there was a submodule there
  # so the link doesn't fail
  rmdir "$subdest" >/dev/null 2>/dev/null || true

  mkdir -p "$(dirname "$subdest")"
  ln -s "$subsrc" -T "$subdest" || echo "Warning: Failed to link submodule to '$subdest'" >&2
done
