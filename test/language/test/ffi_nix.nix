let
  print = msg: builtins.trace msg null;
  append = a: b: a + b;
  toString = builtins.toJSON;
  fileExists =
    f:
        let
            # ./. is relative to this file in the build directory (language/build/dev/nix/ffi_nix.nix)
            # In other targets, this would be relative to 'language/', so let's go back there'
            absPath = if builtins.isString f && builtins.substring 0 1 f != "/" then ./../../../../${f} else f;
        in builtins.pathExists absPath;
  halt = code: if code == 0 then null else builtins.abort (toString code);
  toDynamic = x: x;
in
{ inherit print append toString fileExists halt toDynamic; }
