import { useState, useEffect, useCallback } from "react";

interface UseShortcutRecorderProps {
  onShortcutCaptured: (shortcut: string) => Promise<boolean>;
}

export function useShortcutRecorder({ onShortcutCaptured }: UseShortcutRecorderProps) {
  const [isRecording, setIsRecording] = useState(false);
  const [previewKeys, setPreviewKeys] = useState<string[]>([]);

  const startRecording = useCallback(() => {
    setIsRecording(true);
    setPreviewKeys([]);
  }, []);

  const cancelRecording = useCallback(() => {
    setIsRecording(false);
    setPreviewKeys([]);
  }, []);

  useEffect(() => {
    if (!isRecording) return;

    const handleKeyDown = async (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        cancelRecording();
        return;
      }

      const modifiers: string[] = [];
      if (e.metaKey || e.key === "Meta" || e.key === "OS") {
        modifiers.push("super");
      }
      if (e.ctrlKey || e.key === "Control") {
        modifiers.push("ctrl");
      }
      if (e.altKey || e.key === "Alt") {
        modifiers.push("alt");
      }
      if (e.shiftKey || e.key === "Shift") {
        modifiers.push("shift");
      }

      const isModifierOnly = ["Control", "Alt", "Shift", "Meta", "OS"].includes(e.key);
      if (isModifierOnly) {
        setPreviewKeys(modifiers);
        return;
      }

      let key = e.key.toLowerCase();
      if (key === " ") key = "space";
      else if (key === "arrowup") key = "up";
      else if (key === "arrowdown") key = "down";
      else if (key === "arrowleft") key = "left";
      else if (key === "arrowright") key = "right";

      const parts = [...modifiers];
      if (!parts.includes(key)) {
        parts.push(key);
      }

      if (parts.length === 0) return;

      const formatted = parts.join("+");
      setIsRecording(false);
      setPreviewKeys([]);

      await onShortcutCaptured(formatted);
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isRecording, cancelRecording, onShortcutCaptured]);

  return {
    isRecording,
    previewKeys,
    startRecording,
    cancelRecording,
  };
}
