interface FolderSectionProps {
  imagesPath: string;
  loading: boolean;
  onPickFolder: () => void;
  onOpenFolder: () => void;
}

export function FolderSection({
  imagesPath,
  loading,
  onPickFolder,
  onOpenFolder,
}: FolderSectionProps) {
  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">Wallpaper Folder</h3>
          <p className="card-description">
            Directory where Easy Paper looks for wallpaper images (.png, .jpg, .jpeg, .webp, .bmp).
          </p>
        </div>
      </div>

      <div className="folder-controls-row">
        <input
          type="text"
          className="folder-path-field"
          readOnly
          value={loading ? "Loading directory path..." : imagesPath}
          title={imagesPath}
        />
        <button
          className="btn-primary"
          onClick={onPickFolder}
          disabled={loading}
          type="button"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 8 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />
          </svg>
          <span>Change Folder</span>
        </button>
        <button
          className="btn-secondary"
          onClick={onOpenFolder}
          disabled={loading || !imagesPath}
          type="button"
          title="Open in Windows Explorer"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
          <span>Open</span>
        </button>
      </div>
    </section>
  );
}
