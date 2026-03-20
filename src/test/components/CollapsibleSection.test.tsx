import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CollapsibleSection } from "../../components/develop/controls/CollapsibleSection";

describe("CollapsibleSection component", () => {
  it("should show content by default when defaultOpen is true", () => {
    render(
      <CollapsibleSection title="Test Section" defaultOpen={true}>
        <p>Content here</p>
      </CollapsibleSection>
    );
    expect(screen.getByText("Content here")).toBeTruthy();
  });

  it("should hide content when defaultOpen is false", () => {
    render(
      <CollapsibleSection title="Test Section" defaultOpen={false}>
        <p>Content here</p>
      </CollapsibleSection>
    );
    expect(screen.queryByText("Content here")).toBeNull();
  });

  it("should toggle content on header click", () => {
    render(
      <CollapsibleSection title="Test Section" defaultOpen={true}>
        <p>Content here</p>
      </CollapsibleSection>
    );
    expect(screen.getByText("Content here")).toBeTruthy();
    fireEvent.click(screen.getByText("Test Section"));
    expect(screen.queryByText("Content here")).toBeNull();
    fireEvent.click(screen.getByText("Test Section"));
    expect(screen.getByText("Content here")).toBeTruthy();
  });

  it("should render the title text", () => {
    render(
      <CollapsibleSection title="My Section">
        content
      </CollapsibleSection>
    );
    expect(screen.getByText("My Section")).toBeTruthy();
  });
});
