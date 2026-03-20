import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useDebounce } from "../../hooks/useDebounce";

describe("useDebounce hook", () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });

  it("should debounce the callback", () => {
    const callback = vi.fn();
    const { result } = renderHook(() => useDebounce(callback, 300));
    act(() => { result.current("a"); });
    act(() => { result.current("b"); });
    act(() => { result.current("c"); });
    expect(callback).not.toHaveBeenCalled();
    act(() => { vi.advanceTimersByTime(300); });
    expect(callback).toHaveBeenCalledTimes(1);
    expect(callback).toHaveBeenCalledWith("c");
  });

  it("should call immediately after delay", () => {
    const callback = vi.fn();
    const { result } = renderHook(() => useDebounce(callback, 100));
    act(() => { result.current(); });
    act(() => { vi.advanceTimersByTime(100); });
    expect(callback).toHaveBeenCalledTimes(1);
  });
});
