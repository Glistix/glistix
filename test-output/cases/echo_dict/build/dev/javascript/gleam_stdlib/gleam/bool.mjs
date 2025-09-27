export function and(a, b) {
  return a && b;
}

export function or(a, b) {
  return a || b;
}

export function negate(bool) {
  return !bool;
}

export function nor(a, b) {
  return !(a || b);
}

export function nand(a, b) {
  return !(a && b);
}

export function exclusive_or(a, b) {
  return a !== b;
}

export function exclusive_nor(a, b) {
  return a === b;
}

export function to_string(bool) {
  if (!bool) {
    return "False";
  } else {
    return "True";
  }
}

export function guard(requirement, consequence, alternative) {
  if (requirement) {
    return consequence;
  } else {
    return alternative();
  }
}

export function lazy_guard(requirement, consequence, alternative) {
  if (requirement) {
    return consequence();
  } else {
    return alternative();
  }
}
