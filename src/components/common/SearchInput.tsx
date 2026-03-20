import { useDebounce } from "../../hooks/useDebounce";
import styles from "./SearchInput.module.css";

interface Props { value: string; onChange: (value: string) => void; placeholder?: string; }

export function SearchInput({ value, onChange, placeholder }: Props) {
  const debouncedChange = useDebounce(onChange, 300);
  return (
    <div className={styles.wrapper}>
      <span className={styles.icon}>&#x2315;</span>
      <input className={styles.input} type="text" defaultValue={value} onChange={(e) => debouncedChange(e.target.value)} placeholder={placeholder} />
    </div>
  );
}
