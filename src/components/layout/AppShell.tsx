import { TopToolbar } from "./TopToolbar";
import { StatusBar } from "./StatusBar";
import { FilmStrip } from "./FilmStrip";
import { LibraryView } from "../library/LibraryView";
import { DevelopView } from "../develop/DevelopView";
import { ImportDialog } from "../library/ImportDialog";
import { ExportDialog } from "../export/ExportDialog";
import { useUiStore } from "../../stores/uiStore";
import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import styles from "./AppShell.module.css";

export function AppShell() {
  useKeyboardShortcuts();
  const { viewMode, filmStripOpen, showImportDialog, showExportDialog } = useUiStore();

  return (
    <div className={styles.shell}>
      <TopToolbar />
      <div className={styles.main}>
        {viewMode === "library" ? <LibraryView /> : <DevelopView />}
      </div>
      {filmStripOpen && <FilmStrip />}
      <StatusBar />
      {showImportDialog && <ImportDialog />}
      {showExportDialog && <ExportDialog />}
    </div>
  );
}
