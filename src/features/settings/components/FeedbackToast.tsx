import { FeedbackState } from "../../../types/settings";

interface FeedbackToastProps {
  feedback: FeedbackState | null;
}

export function FeedbackToast({ feedback }: FeedbackToastProps) {
  if (!feedback) return null;

  return (
    <div className={`feedback-toast ${feedback.type}`} role="alert">
      {feedback.type === "success" ? (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      ) : (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
      )}
      <span>{feedback.message}</span>
    </div>
  );
}
