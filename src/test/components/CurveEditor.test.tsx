import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { CurveEditor } from "../../components/develop/controls/CurveEditor";
import type { CurvePoint } from "../../types/develop";

const SIZE = 200;
const PAD = 8;

function mockSvgRect(svg: SVGSVGElement, displayWidth: number, displayHeight: number) {
  vi.spyOn(svg, "getBoundingClientRect").mockReturnValue({
    left: 0,
    top: 0,
    right: displayWidth,
    bottom: displayHeight,
    width: displayWidth,
    height: displayHeight,
    x: 0,
    y: 0,
    toJSON: () => {},
  });
}

describe("CurveEditor", () => {
  const defaultPoints: CurvePoint[] = [
    { x: 0, y: 0 },
    { x: 1, y: 1 },
  ];
  let onChange: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    onChange = vi.fn();
  });

  it("renders SVG with endpoint circles", () => {
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const circles = container.querySelectorAll("circle");
    expect(circles).toHaveLength(2);
  });

  it("renders a path through all points", () => {
    const points: CurvePoint[] = [
      { x: 0, y: 0 },
      { x: 0.5, y: 0.7 },
      { x: 1, y: 1 },
    ];
    const { container } = render(
      <CurveEditor points={points} onChange={onChange} />
    );
    const path = container.querySelector("path");
    expect(path).toBeTruthy();
    const circles = container.querySelectorAll("circle");
    expect(circles).toHaveLength(3);
  });

  it("adds a point on double-click with correct coordinates when SVG is displayed at native size", () => {
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    mockSvgRect(svg, SIZE, SIZE);

    // Click at the center: displayed (100, 100) → SVG (100, 100)
    // curveX = (100 - 8) / 184 ≈ 0.5
    // curveY = 1 - (100 - 8) / 184 ≈ 0.5
    fireEvent.doubleClick(svg, { clientX: 100, clientY: 100 });

    expect(onChange).toHaveBeenCalledTimes(1);
    const newPoints = onChange.mock.calls[0][0] as CurvePoint[];
    expect(newPoints).toHaveLength(3);

    const mid = newPoints[1];
    expect(mid.x).toBeCloseTo(0.5, 1);
    expect(mid.y).toBeCloseTo(0.5, 1);
  });

  it("maps coordinates correctly when SVG is displayed larger than internal size", () => {
    const displayWidth = 400;
    const displayHeight = 400;
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    mockSvgRect(svg, displayWidth, displayHeight);

    // Click at center of displayed SVG (200, 200)
    // svgX = (200 / 400) * 200 = 100, curveX = (100 - 8) / 184 ≈ 0.5
    fireEvent.doubleClick(svg, { clientX: 200, clientY: 200 });

    expect(onChange).toHaveBeenCalledTimes(1);
    const newPoints = onChange.mock.calls[0][0] as CurvePoint[];
    const mid = newPoints[1];
    expect(mid.x).toBeCloseTo(0.5, 1);
    expect(mid.y).toBeCloseTo(0.5, 1);
  });

  it("maps coordinates correctly when SVG is displayed at 300px wide", () => {
    const displayWidth = 300;
    const displayHeight = 300;
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    mockSvgRect(svg, displayWidth, displayHeight);

    // Click at 75% across displayed SVG (225, 75)
    // svgX = (225 / 300) * 200 = 150
    // curveX = (150 - 8) / 184 ≈ 0.77
    // svgY = (75 / 300) * 200 = 50
    // curveY = 1 - (50 - 8) / 184 ≈ 0.77
    fireEvent.doubleClick(svg, { clientX: 225, clientY: 75 });

    expect(onChange).toHaveBeenCalledTimes(1);
    const newPoints = onChange.mock.calls[0][0] as CurvePoint[];
    const mid = newPoints[1];
    expect(mid.x).toBeCloseTo(0.77, 1);
    expect(mid.y).toBeCloseTo(0.77, 1);
  });

  it("clamps coordinates to [0, 1] range", () => {
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    mockSvgRect(svg, SIZE, SIZE);

    // Click beyond the bottom-right padding area
    fireEvent.doubleClick(svg, { clientX: SIZE + 10, clientY: SIZE + 10 });

    expect(onChange).toHaveBeenCalledTimes(1);
    const newPoints = onChange.mock.calls[0][0] as CurvePoint[];
    const added = newPoints[newPoints.length - 1];
    expect(added.x).toBeLessThanOrEqual(1);
    expect(added.y).toBeGreaterThanOrEqual(0);
  });

  it("constrains first point to x=0 and last point to x=1 during drag", () => {
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    const circles = container.querySelectorAll("circle");
    mockSvgRect(svg, SIZE, SIZE);

    // Start dragging the first point
    fireEvent.mouseDown(circles[0], { preventDefault: () => {} });

    // Drag to center — first point should stay at x=0
    fireEvent.mouseMove(svg, { clientX: 100, clientY: 50 });

    expect(onChange).toHaveBeenCalledTimes(1);
    const movedPoints = onChange.mock.calls[0][0] as CurvePoint[];
    expect(movedPoints[0].x).toBe(0);
    expect(movedPoints[0].y).toBeGreaterThan(0);
  });

  it("inserts new points sorted by x coordinate", () => {
    const { container } = render(
      <CurveEditor points={defaultPoints} onChange={onChange} />
    );
    const svg = container.querySelector("svg")!;
    mockSvgRect(svg, SIZE, SIZE);

    // Add point at x ≈ 0.25
    const qX = PAD + 0.25 * (SIZE - 2 * PAD);
    fireEvent.doubleClick(svg, { clientX: qX, clientY: 100 });

    const newPoints = onChange.mock.calls[0][0] as CurvePoint[];
    expect(newPoints).toHaveLength(3);
    expect(newPoints[0].x).toBe(0);
    expect(newPoints[1].x).toBeCloseTo(0.25, 1);
    expect(newPoints[2].x).toBe(1);
  });
});
