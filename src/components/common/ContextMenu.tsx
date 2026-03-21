import { useEffect, useRef, type ReactNode } from "react";
import styles from "./ContextMenu.module.css";

export interface MenuPosition {
  x: number;
  y: number;
}

interface ContextMenuProps {
  position: MenuPosition;
  onClose: () => void;
  children: ReactNode;
}

export function ContextMenu({ position, onClose, children }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("mousedown", handleClick);
    window.addEventListener("keydown", handleKey);
    return () => {
      window.removeEventListener("mousedown", handleClick);
      window.removeEventListener("keydown", handleKey);
    };
  }, [onClose]);

  // Clamp to viewport
  useEffect(() => {
    const menu = menuRef.current;
    if (!menu) return;
    const rect = menu.getBoundingClientRect();
    if (rect.right > window.innerWidth) {
      menu.style.left = `${window.innerWidth - rect.width - 4}px`;
    }
    if (rect.bottom > window.innerHeight) {
      menu.style.top = `${window.innerHeight - rect.height - 4}px`;
    }
  }, [position]);

  return (
    <div
      ref={menuRef}
      className={styles.menu}
      style={{ left: position.x, top: position.y }}
    >
      {children}
    </div>
  );
}

interface MenuItemProps {
  label: string;
  onClick: () => void;
  danger?: boolean;
}

export function MenuItem({ label, onClick, danger }: MenuItemProps) {
  return (
    <button
      className={`${styles.item} ${danger ? styles.danger : ""}`}
      onClick={onClick}
    >
      {label}
    </button>
  );
}

interface SubMenuProps {
  label: string;
  children: ReactNode;
}

export function SubMenu({ label, children }: SubMenuProps) {
  return (
    <div className={styles.submenu}>
      <div className={styles.submenuLabel}>
        {label}
        <span className={styles.arrow}>&#9654;</span>
      </div>
      <div className={styles.submenuContent}>{children}</div>
    </div>
  );
}

export function MenuDivider() {
  return <div className={styles.divider} />;
}
