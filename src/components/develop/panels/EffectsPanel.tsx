import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";

export function EffectsPanel() {
  const { editParams, updateParam } = useDevelopStore();
  return (
    <CollapsibleSection title="Effects" defaultOpen={false}>
      <AdjustmentSlider label="Dehaze" value={editParams.dehaze} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("dehaze", v)} />
      <AdjustmentSlider label="Vignette" value={editParams.vignette_amount} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("vignette_amount", v)} />
      <AdjustmentSlider label="Grain Amount" value={editParams.grain_amount} min={0} max={100} defaultValue={0} onChange={(v) => updateParam("grain_amount", v)} />
      <AdjustmentSlider label="Grain Size" value={editParams.grain_size} min={0} max={100} defaultValue={25} onChange={(v) => updateParam("grain_size", v)} />
    </CollapsibleSection>
  );
}
