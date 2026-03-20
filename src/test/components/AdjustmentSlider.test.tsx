import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AdjustmentSlider } from "../../components/develop/controls/AdjustmentSlider";

describe("AdjustmentSlider component", () => {
  it("should render label and value", () => {
    const onChange = vi.fn();
    render(
      <AdjustmentSlider label="Exposure" value={1.5} min={-5} max={5} step={0.05} onChange={onChange} />
    );
    expect(screen.getByText("Exposure")).toBeTruthy();
    expect(screen.getByText("1.5")).toBeTruthy();
  });

  it("should display integer value for integer step", () => {
    const onChange = vi.fn();
    render(
      <AdjustmentSlider label="Contrast" value={50} min={-100} max={100} step={1} onChange={onChange} />
    );
    expect(screen.getByText("50")).toBeTruthy();
  });

  it("should call onChange when slider changes", () => {
    const onChange = vi.fn();
    render(
      <AdjustmentSlider label="Test" value={0} min={-100} max={100} onChange={onChange} />
    );
    const slider = screen.getByRole("slider");
    fireEvent.change(slider, { target: { value: "50" } });
    expect(onChange).toHaveBeenCalledWith(50);
  });

  it("should reset to default on double-click", () => {
    const onChange = vi.fn();
    render(
      <AdjustmentSlider label="Test" value={50} min={-100} max={100} defaultValue={0} onChange={onChange} />
    );
    const slider = screen.getByRole("slider");
    fireEvent.doubleClick(slider);
    expect(onChange).toHaveBeenCalledWith(0);
  });
});
