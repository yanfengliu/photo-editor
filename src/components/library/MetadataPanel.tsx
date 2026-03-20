import type { ImageRecord } from "../../types/catalog";
import { Rating } from "../common/Rating";
import { ColorLabel } from "../common/ColorLabel";
import { FlagToggle } from "../common/FlagToggle";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./MetadataPanel.module.css";

interface Props { image: ImageRecord | null; }

export function MetadataPanel({ image }: Props) {
  const { setRating, setColorLabel, setFlag } = useCatalogStore();
  if (!image) return <div className={styles.panel}><div className={styles.empty}>No image selected</div></div>;

  return (
    <div className={styles.panel}>
      <div className={styles.section}>
        <h3 className={styles.sectionTitle}>Quick Develop</h3>
        <div className={styles.row}><span className={styles.label}>Rating</span><Rating value={image.rating} onChange={(r) => setRating(image.id, r)} /></div>
        <div className={styles.row}><span className={styles.label}>Color</span><ColorLabel value={image.color_label} onChange={(c) => setColorLabel(image.id, c)} /></div>
        <div className={styles.row}><span className={styles.label}>Flag</span><FlagToggle value={image.flag} onChange={(f) => setFlag(image.id, f)} /></div>
      </div>
      <div className={styles.section}>
        <h3 className={styles.sectionTitle}>File Info</h3>
        <div className={styles.metaRow}><span className={styles.metaLabel}>File</span><span className={styles.metaValue}>{image.file_name}</span></div>
        <div className={styles.metaRow}><span className={styles.metaLabel}>Format</span><span className={styles.metaValue}>{image.format.toUpperCase()}</span></div>
        <div className={styles.metaRow}><span className={styles.metaLabel}>Size</span><span className={styles.metaValue}>{image.width} x {image.height}</span></div>
        {image.date_taken && <div className={styles.metaRow}><span className={styles.metaLabel}>Date</span><span className={styles.metaValue}>{image.date_taken}</span></div>}
      </div>
      {(image.camera || image.lens) && (
        <div className={styles.section}>
          <h3 className={styles.sectionTitle}>Camera</h3>
          {image.camera && <div className={styles.metaRow}><span className={styles.metaLabel}>Camera</span><span className={styles.metaValue}>{image.camera}</span></div>}
          {image.lens && <div className={styles.metaRow}><span className={styles.metaLabel}>Lens</span><span className={styles.metaValue}>{image.lens}</span></div>}
          {image.iso && <div className={styles.metaRow}><span className={styles.metaLabel}>ISO</span><span className={styles.metaValue}>{image.iso}</span></div>}
          {image.focal_length && <div className={styles.metaRow}><span className={styles.metaLabel}>Focal</span><span className={styles.metaValue}>{image.focal_length}mm</span></div>}
          {image.aperture && <div className={styles.metaRow}><span className={styles.metaLabel}>Aperture</span><span className={styles.metaValue}>f/{image.aperture}</span></div>}
          {image.shutter_speed && <div className={styles.metaRow}><span className={styles.metaLabel}>Shutter</span><span className={styles.metaValue}>{image.shutter_speed}s</span></div>}
        </div>
      )}
      {image.tags.length > 0 && (
        <div className={styles.section}>
          <h3 className={styles.sectionTitle}>Tags</h3>
          <div className={styles.tags}>{image.tags.map((tag) => <span key={tag} className={styles.tag}>{tag}</span>)}</div>
        </div>
      )}
    </div>
  );
}
