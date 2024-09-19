import { DependencyList, MutableRefObject, useCallback } from "react";

export function useLockCallback(
  callback: () => Promise<void>,
  ref: MutableRefObject<boolean>,
  deps: DependencyList
) {
  return useCallback(() => {
    if (ref.current) {
      return;
    }
    ref.current = true;
    callback().finally(() => (ref.current = false));
  }, deps);
}
