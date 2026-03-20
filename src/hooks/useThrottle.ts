import { useRef, useCallback, useEffect } from "react";

export function useThrottle<T extends (...args: any[]) => void>(
  callback: T,
  delay: number
): T {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const callbackRef = useRef(callback);
  const lastArgsRef = useRef<Parameters<T> | null>(null);
  const lastInvokeTimeRef = useRef<number | null>(null);
  callbackRef.current = callback;

  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  return useCallback(
    (...args: Parameters<T>) => {
      const now = Date.now();
      lastArgsRef.current = args;

      if (
        lastInvokeTimeRef.current === null ||
        now - lastInvokeTimeRef.current >= delay
      ) {
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
          timeoutRef.current = null;
        }
        lastInvokeTimeRef.current = now;
        callbackRef.current(...args);
        return;
      }

      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      timeoutRef.current = setTimeout(() => {
        timeoutRef.current = null;
        lastInvokeTimeRef.current = Date.now();
        const argsToFlush = lastArgsRef.current;
        if (argsToFlush) callbackRef.current(...argsToFlush);
      }, delay - (now - lastInvokeTimeRef.current));
    },
    [delay]
  ) as T;
}
