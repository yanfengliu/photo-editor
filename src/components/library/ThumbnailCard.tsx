import type { ImageRecord } from "../../types/catalog";
import { Rating } from "../common/Rating";
import { FlagToggle } from "../common/FlagToggle";
import styles from "./ThumbnailCard.module.css";

interface Props { image: ImageRecord; isSelected: boolean; onClick: () => void; onDoubleClick: () => void; }

export function ThumbnailCard({ image, isSelected, onClick, onDoubleClick }: Props) {
  return (
    <div className={`${styles.card} ${isSelected ? styles.selected : ""}`} onClick={onClick} onDoubleClick={onDoubleClick}>
      <div className={styles.preview}>
        <div className={styles.placeholder}><span className={styles.format}>{image.format.toUpperCase()}</span></div>
      </div>
      <div className={styles.info}>
        <span className={styles.name}>{image.file_name}</span>
        <div className={styles.meta}>
          <Rating value={image.rating} size="small" onChange={() => {}} />
          <FlagToggle value={image.flag} onChange={() => {}} size="small" />
        </div>
      </div>
      {image.color_label !== "none" && <div className={styles.colorLabel} style={{ backgroundColor: `var(--color-${image.color_label})` }} />}
    </div>
  );
}
