import { useEffect } from "react";
import { ImageCanvas } from "./ImageCanvas";
import { BasicAdjustments } from "./panels/BasicAdjustments";
import { WhiteBalance } from "./panels/WhiteBalance";
import { ToneCurve } from "./panels/ToneCurve";
import { HSLPanel } from "./panels/HSLPanel";
import { DetailPanel } from "./panels/DetailPanel";
import { EffectsPanel } from "./panels/EffectsPanel";
import { useUiStore } from "../../stores/uiStore";
import { useDevelopStore } from "../../stores/developStore";
import styles from "./DevelopView.module.css";

export function DevelopView() {
  const { selectedImageId, rightPanelOpen } = useUiStore();
  const { setCurrentImage, currentImageId } = useDevelopStore();
  useEffect(() => { if (selectedImageId && selectedImageId !== currentImageId) setCurrentImage(selectedImageId); }, [selectedImageId]);

  return (
    <div className={styles.develop}>
      <div className={styles.center}>{selectedImageId ? <ImageCanvas /> : <div className={styles.empty}>Select an image to edit</div>}</div>
      {rightPanelOpen && <div className={styles.rightPanel}><div className={styles.panels}><WhiteBalance /><BasicAdjustments /><ToneCurve /><HSLPanel /><DetailPanel /><EffectsPanel /></div></div>}
    </div>
  );
}
