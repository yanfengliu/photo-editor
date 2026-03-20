import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useThrottle } from "../../hooks/useThrottle";

describe("useThrottle hook", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should call immediately and then flush the latest value after the throttle window", () => {
    const callback = vi.fn();
    const { result } = renderHook(() => useThrottle(callback, 30));

    act(() => {
      result.current("a");
      result.current("b");
      result.current("c");
    });

    expect(callback).toHaveBeenCalledTimes(1);
    expect(callback).toHaveBeenNthCalledWith(1, "a");

    act(() => {
      vi.advanceTimersByTime(30);
    });

    expect(callback).toHaveBeenCalledTimes(2);
    expect(callback).toHaveBeenNthCalledWith(2, "c");
  });
});
