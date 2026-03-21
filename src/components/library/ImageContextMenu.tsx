import { useCallback, useState } from "react";
import {
  ContextMenu,
  MenuItem,
  SubMenu,
  MenuDivider,
  type MenuPosition,
} from "../common/ContextMenu";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./ImageContextMenu.module.css";

interface Props {
  position: MenuPosition;
  imageId: string;
  onClose: () => void;
}

export function ImageContextMenu({ position, imageId, onClose }: Props) {
  const { setViewMode, selectImage, selectedImageIds, setShowDeleteConfirm } =
    useUiStore();
  const { collections, addToCollection, createCollection, setRating, setFlag } =
    useCatalogStore();
  const [showNewColl, setShowNewColl] = useState(false);
  const [newCollName, setNewCollName] = useState("");

  const targetIds =
    selectedImageIds.length > 0 && selectedImageIds.includes(imageId)
      ? selectedImageIds
      : [imageId];

  const handleDevelop = useCallback(() => {
    selectImage(imageId);
    setViewMode("develop");
    onClose();
  }, [imageId, selectImage, setViewMode, onClose]);

  const handleRate = useCallback(
    (rating: number) => {
      for (const id of targetIds) setRating(id, rating);
      onClose();
    },
    [targetIds, setRating, onClose]
  );

  const handleFlag = useCallback(
    (flag: string) => {
      for (const id of targetIds) setFlag(id, flag);
      onClose();
    },
    [targetIds, setFlag, onClose]
  );

  const handleAddToCollection = useCallback(
    (collectionId: string) => {
      addToCollection(collectionId, targetIds);
      onClose();
    },
    [targetIds, addToCollection, onClose]
  );

  const handleCreateAndAdd = useCallback(async () => {
    const name = newCollName.trim();
    if (!name) return;
    await createCollection(name);
    // Get the newly created collection (last in list)
    const cols = useCatalogStore.getState().collections;
    const newCol = cols[cols.length - 1];
    if (newCol) {
      await addToCollection(newCol.id, targetIds);
    }
    onClose();
  }, [newCollName, createCollection, addToCollection, targetIds, onClose]);

  const handleDelete = useCallback(() => {
    // Ensure targeted images are selected
    if (!selectedImageIds.includes(imageId)) {
      selectImage(imageId);
    }
    setShowDeleteConfirm(true);
    onClose();
  }, [imageId, selectedImageIds, selectImage, setShowDeleteConfirm, onClose]);

  return (
    <ContextMenu position={position} onClose={onClose}>
      <MenuItem label="Edit in Develop" onClick={handleDevelop} />
      <MenuDivider />
      <SubMenu label="Set Rating">
        {[0, 1, 2, 3, 4, 5].map((r) => (
          <MenuItem
            key={r}
            label={r === 0 ? "No Rating" : "★".repeat(r)}
            onClick={() => handleRate(r)}
          />
        ))}
      </SubMenu>
      <SubMenu label="Set Flag">
        <MenuItem label="Picked" onClick={() => handleFlag("picked")} />
        <MenuItem label="Rejected" onClick={() => handleFlag("rejected")} />
        <MenuItem label="Unflagged" onClick={() => handleFlag("none")} />
      </SubMenu>
      <MenuDivider />
      <SubMenu label="Add to Collection">
        {collections.map((col) => (
          <MenuItem
            key={col.id}
            label={col.name}
            onClick={() => handleAddToCollection(col.id)}
          />
        ))}
        {collections.length > 0 && <MenuDivider />}
        {showNewColl ? (
          <div className={styles.newCollRow}>
            <input
              className={styles.newCollInput}
              type="text"
              placeholder="Name..."
              value={newCollName}
              onChange={(e) => setNewCollName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreateAndAdd();
                if (e.key === "Escape") setShowNewColl(false);
                e.stopPropagation();
              }}
              onClick={(e) => e.stopPropagation()}
              autoFocus
            />
          </div>
        ) : (
          <MenuItem label="+ New Collection" onClick={() => setShowNewColl(true)} />
        )}
      </SubMenu>
      <MenuDivider />
      <MenuItem
        label={
          targetIds.length > 1
            ? `Delete ${targetIds.length} Images`
            : "Delete Image"
        }
        onClick={handleDelete}
        danger
      />
    </ContextMenu>
  );
}
