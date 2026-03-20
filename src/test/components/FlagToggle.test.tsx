import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FlagToggle } from "../../components/common/FlagToggle";

describe("FlagToggle component", () => {
  it("should render pick and reject buttons", () => {
    const onChange = vi.fn();
    render(<FlagToggle value="none" onChange={onChange} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(2);
  });

  it("should toggle to picked when clicking pick button", () => {
    const onChange = vi.fn();
    render(<FlagToggle value="none" onChange={onChange} />);
    fireEvent.click(screen.getByTitle("Pick (P)"));
    expect(onChange).toHaveBeenCalledWith("picked");
  });

  it("should toggle to none when clicking picked again", () => {
    const onChange = vi.fn();
    render(<FlagToggle value="picked" onChange={onChange} />);
    fireEvent.click(screen.getByTitle("Pick (P)"));
    expect(onChange).toHaveBeenCalledWith("none");
  });

  it("should toggle to rejected", () => {
    const onChange = vi.fn();
    render(<FlagToggle value="none" onChange={onChange} />);
    fireEvent.click(screen.getByTitle("Reject (X)"));
    expect(onChange).toHaveBeenCalledWith("rejected");
  });

  it("should show picked state visually", () => {
    const onChange = vi.fn();
    const { container } = render(<FlagToggle value="picked" onChange={onChange} />);
    expect(container.querySelector(".picked")).toBeTruthy();
  });

  it("should show rejected state visually", () => {
    const onChange = vi.fn();
    const { container } = render(<FlagToggle value="rejected" onChange={onChange} />);
    expect(container.querySelector(".rejected")).toBeTruthy();
  });
});
