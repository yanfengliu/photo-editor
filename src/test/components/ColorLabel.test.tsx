import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ColorLabel } from "../../components/common/ColorLabel";

describe("ColorLabel component", () => {
  it("should render 6 color buttons (none + 5 colors)", () => {
    const onChange = vi.fn();
    render(<ColorLabel value="none" onChange={onChange} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(6);
  });

  it("should call onChange with color when clicked", () => {
    const onChange = vi.fn();
    render(<ColorLabel value="none" onChange={onChange} />);
    fireEvent.click(screen.getByTitle("red"));
    expect(onChange).toHaveBeenCalledWith("red");
  });

  it("should show active state for selected color", () => {
    const onChange = vi.fn();
    const { container } = render(<ColorLabel value="blue" onChange={onChange} />);
    const active = container.querySelectorAll(".active");
    expect(active).toHaveLength(1);
  });
});
