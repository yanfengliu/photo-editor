import { useEffect } from "react";
import { useDevelopStore } from "../../../stores/developStore";
import styles from "./HistoryPanel.module.css";

export function HistoryPanel() {
  const { history, loadHistory, currentImageId, undoStack, undo, resetEdits } = useDevelopStore();
  useEffect(() => { if (currentImageId) loadHistory(); }, [currentImageId]);

  return (
    <div className={styles.panel}>
      <div className={styles.section}>
        <h3 className={styles.title}>History</h3>
        <div className={styles.actions}>
          <button className={styles.actionBtn} onClick={() => undo()} disabled={undoStack.length === 0}>Undo</button>
          <button className={styles.actionBtn} onClick={() => resetEdits()}>Reset</button>
        </div>
      </div>
      <div className={styles.list}>
        {history.length === 0 && <div className={styles.empty}>No history yet</div>}
        {history.map((entry) => (
          <div key={entry.id} className={styles.entry}>
            <span className={styles.action}>{entry.action}</span>
            <span className={styles.time}>{new Date(entry.created_at).toLocaleTimeString()}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
