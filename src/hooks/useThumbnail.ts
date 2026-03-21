import { useEffect, useState } from "react";
import { loadThumbnail } from "../api/image";

export function useThumbnail(imageId: string): string | null {
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let objectUrl: string | null = null;

    setUrl(null);

    loadThumbnail(imageId)
      .then((bytes) => {
        if (!active || bytes.length === 0) return;
        const buffer = new ArrayBuffer(bytes.byteLength);
        new Uint8Array(buffer).set(bytes);
        objectUrl = URL.createObjectURL(
          new Blob([buffer], { type: "image/jpeg" })
        );
        setUrl(objectUrl);
      })
      .catch(() => {
        setUrl(null);
      });

    return () => {
      active = false;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [imageId]);

  return url;
}
