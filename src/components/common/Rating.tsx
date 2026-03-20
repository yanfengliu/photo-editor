import styles from "./Rating.module.css";

interface Props { value: number; onChange: (rating: number) => void; size?: "small" | "normal"; }

export function Rating({ value, onChange, size = "normal" }: Props) {
  return (
    <div className={`${styles.rating} ${size === "small" ? styles.small : ""}`}>
      {[1,2,3,4,5].map((star) => (
        <button key={star} className={`${styles.star} ${star <= value ? styles.filled : ""}`} onClick={(e) => { e.stopPropagation(); onChange(star === value ? 0 : star); }}>★</button>
      ))}
    </div>
  );
}
