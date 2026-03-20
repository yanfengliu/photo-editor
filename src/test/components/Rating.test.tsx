import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Rating } from "../../components/common/Rating";

describe("Rating component", () => {
  it("should render 5 star buttons", () => {
    const onChange = vi.fn();
    render(<Rating value={0} onChange={onChange} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(5);
  });

  it("should show filled stars based on value", () => {
    const onChange = vi.fn();
    const { container } = render(<Rating value={3} onChange={onChange} />);
    const filled = container.querySelectorAll(".filled");
    expect(filled).toHaveLength(3);
  });

  it("should call onChange with star number when clicked", () => {
    const onChange = vi.fn();
    render(<Rating value={0} onChange={onChange} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[2]); // 3rd star
    expect(onChange).toHaveBeenCalledWith(3);
  });

  it("should toggle off when clicking current rating", () => {
    const onChange = vi.fn();
    render(<Rating value={3} onChange={onChange} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[2]); // Click 3rd star again
    expect(onChange).toHaveBeenCalledWith(0);
  });

  it("should apply small size class", () => {
    const onChange = vi.fn();
    const { container } = render(<Rating value={0} onChange={onChange} size="small" />);
    expect(container.querySelector(".small")).toBeTruthy();
  });
});
