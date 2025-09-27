export function flip(fun) {
  return (b, a) => { return fun(a, b); };
}

export function identity(x) {
  return x;
}

export function tap(arg, effect) {
  effect(arg);
  return arg;
}
