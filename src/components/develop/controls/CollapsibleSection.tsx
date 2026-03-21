import { useState, type ReactNode } from "react";
import styles from "./CollapsibleSection.module.css";

interface Props { title: string; defaultOpen?: boolean; onToggle?: (open: boolean) => void; children: ReactNode; }

export function CollapsibleSection({ title, defaultOpen = true, onToggle, children }: Props) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const toggle = () => {
    const next = !isOpen;
    setIsOpen(next);
    onToggle?.(next);
  };
  return (
    <div className={styles.section}>
      <button className={styles.header} onClick={toggle}>
        <span className={`${styles.arrow} ${isOpen ? styles.open : ""}`}>&#9654;</span>
        <span className={styles.title}>{title}</span>
      </button>
      {isOpen && <div className={styles.content}>{children}</div>}
    </div>
  );
}
