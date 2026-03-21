import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ContextMenu, MenuItem, SubMenu, MenuDivider } from "../../components/common/ContextMenu";

describe("ContextMenu", () => {
  const onClose = vi.fn();

  beforeEach(() => {
    onClose.mockClear();
  });

  it("should render menu items", () => {
    render(
      <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
        <MenuItem label="Edit" onClick={() => {}} />
        <MenuItem label="Delete" onClick={() => {}} danger />
      </ContextMenu>
    );
    expect(screen.getByText("Edit")).toBeTruthy();
    expect(screen.getByText("Delete")).toBeTruthy();
  });

  it("should call onClick when item is clicked", () => {
    const onClick = vi.fn();
    render(
      <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
        <MenuItem label="Edit" onClick={onClick} />
      </ContextMenu>
    );
    fireEvent.click(screen.getByText("Edit"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("should close on Escape key", () => {
    render(
      <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
        <MenuItem label="Edit" onClick={() => {}} />
      </ContextMenu>
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("should close on outside click", () => {
    render(
      <div>
        <div data-testid="outside">outside</div>
        <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
          <MenuItem label="Edit" onClick={() => {}} />
        </ContextMenu>
      </div>
    );
    fireEvent.mouseDown(screen.getByTestId("outside"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("should render submenu", () => {
    render(
      <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
        <SubMenu label="Rating">
          <MenuItem label="★★★" onClick={() => {}} />
        </SubMenu>
      </ContextMenu>
    );
    expect(screen.getByText("Rating")).toBeTruthy();
    expect(screen.getByText("★★★")).toBeTruthy();
  });

  it("should render divider", () => {
    const { container } = render(
      <ContextMenu position={{ x: 100, y: 200 }} onClose={onClose}>
        <MenuItem label="Edit" onClick={() => {}} />
        <MenuDivider />
        <MenuItem label="Delete" onClick={() => {}} />
      </ContextMenu>
    );
    expect(container.querySelector('[class*="divider"]')).toBeTruthy();
  });

  it("should position at given coordinates", () => {
    const { container } = render(
      <ContextMenu position={{ x: 150, y: 300 }} onClose={onClose}>
        <MenuItem label="Edit" onClick={() => {}} />
      </ContextMenu>
    );
    const menu = container.querySelector('[class*="menu"]') as HTMLElement;
    expect(menu.style.left).toBe("150px");
    expect(menu.style.top).toBe("300px");
  });
});
