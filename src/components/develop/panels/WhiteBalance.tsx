import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";

export function WhiteBalance() {
  const { editParams, updateParam } = useDevelopStore();
  return (
    <CollapsibleSection title="White Balance">
      <AdjustmentSlider label="Temperature" value={editParams.temperature} min={2000} max={12000} step={100} defaultValue={6500} onChange={(v) => updateParam("temperature", v)} />
      <AdjustmentSlider label="Tint" value={editParams.tint} min={-150} max={150} defaultValue={0} onChange={(v) => updateParam("tint", v)} />
    </CollapsibleSection>
  );
}
