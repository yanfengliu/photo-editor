import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";

export function BasicAdjustments() {
  const { editParams, updateParam } = useDevelopStore();
  return (
    <CollapsibleSection title="Basic">
      <AdjustmentSlider label="Exposure" value={editParams.exposure} min={-5} max={5} step={0.05} defaultValue={0} onChange={(v) => updateParam("exposure", v)} />
      <AdjustmentSlider label="Contrast" value={editParams.contrast} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("contrast", v)} />
      <AdjustmentSlider label="Highlights" value={editParams.highlights} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("highlights", v)} />
      <AdjustmentSlider label="Shadows" value={editParams.shadows} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("shadows", v)} />
      <AdjustmentSlider label="Whites" value={editParams.whites} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("whites", v)} />
      <AdjustmentSlider label="Blacks" value={editParams.blacks} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("blacks", v)} />
      <AdjustmentSlider label="Saturation" value={editParams.saturation} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("saturation", v)} />
      <AdjustmentSlider label="Vibrance" value={editParams.vibrance} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("vibrance", v)} />
    </CollapsibleSection>
  );
}
