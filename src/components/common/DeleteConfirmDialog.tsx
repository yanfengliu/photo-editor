import { useCallback } from "react";
import { Modal } from "./Modal";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./DeleteConfirmDialog.module.css";

export function DeleteConfirmDialog() {
  const { selectedImageIds, setShowDeleteConfirm, selectImage } = useUiStore();
  const { deleteImages } = useCatalogStore();

  const count = selectedImageIds.length;

  const handleClose = useCallback(() => {
    setShowDeleteConfirm(false);
  }, [setShowDeleteConfirm]);

  const handleDelete = useCallback(async () => {
    if (selectedImageIds.length === 0) return;
    await deleteImages(selectedImageIds);
    selectImage(null);
    setShowDeleteConfirm(false);
  }, [selectedImageIds, deleteImages, selectImage, setShowDeleteConfirm]);

  return (
    <Modal title="Delete Images" onClose={handleClose}>
      <p className={styles.message}>
        Are you sure you want to remove {count === 1 ? "this image" : `these ${count} images`} from
        the catalog? The original files will not be deleted.
      </p>
      <div className={styles.actions}>
        <button className={styles.cancelBtn} onClick={handleClose}>
          Cancel
        </button>
        <button className={styles.deleteBtn} onClick={handleDelete}>
          Delete
        </button>
      </div>
    </Modal>
  );
}
