import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Modal } from "../../components/common/Modal";

describe("Modal component", () => {
  it("should render title and children", () => {
    const onClose = vi.fn();
    render(
      <Modal title="Test Modal" onClose={onClose}>
        <p>Modal content</p>
      </Modal>
    );
    expect(screen.getByText("Test Modal")).toBeTruthy();
    expect(screen.getByText("Modal content")).toBeTruthy();
  });

  it("should call onClose when close button clicked", () => {
    const onClose = vi.fn();
    render(
      <Modal title="Test" onClose={onClose}>
        content
      </Modal>
    );
    fireEvent.click(screen.getByText("\u00d7"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should call onClose when overlay clicked", () => {
    const onClose = vi.fn();
    const { container } = render(
      <Modal title="Test" onClose={onClose}>
        content
      </Modal>
    );
    const overlay = container.firstChild as HTMLElement;
    fireEvent.click(overlay);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should not call onClose when modal body clicked", () => {
    const onClose = vi.fn();
    render(
      <Modal title="Test" onClose={onClose}>
        <p>content</p>
      </Modal>
    );
    fireEvent.click(screen.getByText("content"));
    expect(onClose).not.toHaveBeenCalled();
  });

  it("should close on Escape key", () => {
    const onClose = vi.fn();
    render(
      <Modal title="Test" onClose={onClose}>
        content
      </Modal>
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
