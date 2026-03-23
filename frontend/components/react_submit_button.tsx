import React, { useState } from "react";

/**
 * @title React Submit Button States
 * @notice Typed submit button with a strict state machine, safe label handling,
 *         double-submit prevention, and ARIA accessibility semantics.
 * @dev Labels are rendered as React text nodes — no dangerouslySetInnerHTML path.
 *      Interaction is blocked while submitting to prevent duplicate blockchain transactions.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
 * @notice All supported visual/interaction states.
 * @dev Allowed transitions:
 *   idle        → submitting | disabled
 *   submitting  → success | error | disabled
 *   success     → idle | disabled
 *   error       → idle | submitting | disabled
 *   disabled    → idle
import React from "react";

/**
 * @title React Submit Button Component States
 * @notice Centralized submit-button state model for consistent UX and safe defaults.
 * @dev Uses strict union types and deterministic state mapping to reduce misuse.
 */
export type SubmitButtonState = "idle" | "submitting" | "success" | "error" | "disabled";

/**
 * @notice Optional per-state label overrides.
 * @dev Values are normalized: non-strings, empty strings, and control characters are rejected.
 * @title Submit Button Labels
 * @notice Optional custom labels for each user-visible state.
 */
export interface SubmitButtonLabels {
  idle?: string;
  submitting?: string;
  success?: string;
  error?: string;
  disabled?: string;
}

/**
 * @notice Props accepted by ReactSubmitButton.
 * @param state          Current button state (required).
 * @param previousState  Previous state used for strict transition validation.
 * @param strictTransitions  When true, invalid state jumps fall back to previousState.
 * @param labels         Optional label overrides per state.
 * @param onClick        Async-safe click handler; blocked while submitting/disabled.
 * @param className      Additional CSS class.
 * @param id             HTML id attribute.
 * @param type           HTML button type. Default: "button".
 * @param disabled       External disabled override.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  previousState?: SubmitButtonState;
  strictTransitions?: boolean;
  labels?: SubmitButtonLabels;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  className?: string;
  id?: string;
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
}

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_LABEL_LENGTH = 80;
const CONTROL_CHAR_RE = /[\u0000-\u001F\u007F]/g;

 * @title Submit Button Props
 * @notice Defines behavior, labeling, and accessibility requirements.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  labels?: SubmitButtonLabels;
  onClick?: () => void | Promise<void>;
  className?: string;
  id?: string;
  type?: "button" | "submit";
  disabled?: boolean;
}

const DEFAULT_LABELS: Required<SubmitButtonLabels> = {
  idle: "Submit",
  submitting: "Submitting...",
  success: "Submitted",
  error: "Try Again",
  disabled: "Submit Disabled",
};

/**
 * @notice Allowed state transitions.
 * @dev Centralised here so both the guard and tests share one source of truth.
 */
export const ALLOWED_TRANSITIONS: Record<SubmitButtonState, SubmitButtonState[]> = {
  idle: ["submitting", "disabled"],
  submitting: ["success", "error", "disabled"],
  success: ["idle", "disabled"],
  error: ["idle", "submitting", "disabled"],
  disabled: ["idle"],
};
 * @title Safe Label Resolver
 * @notice Provides a bounded, non-empty label to avoid empty UI text states.
 * @dev React escapes text content by default; this function only normalizes.
 */
export function resolveSubmitButtonLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels,
): string {
  const candidate = labels?.[state];

  if (typeof candidate !== "string") {
    return DEFAULT_LABELS[state];
  }

  const normalized = candidate.trim();
  if (!normalized) {
    return DEFAULT_LABELS[state];
  }

  return normalized.length > 80 ? `${normalized.slice(0, 77)}...` : normalized;
}

/**
 * @title Disabled Guard
 * @notice Computes final disabled state from explicit disabled flag and workflow state.
 */
export function isSubmitButtonDisabled(state: SubmitButtonState, disabled?: boolean): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting";
}

/**
 * @title Aria Busy Guard
 * @notice Announces loading semantics only during active submission.
 */
export function isSubmitButtonBusy(state: SubmitButtonState): boolean {
  return state === "submitting";
}

const BASE_STYLE: React.CSSProperties = {
  minHeight: "44px",
  minWidth: "120px",
  borderRadius: "8px",
  border: "1px solid #4f46e5",
  padding: "0.5rem 1rem",
  color: "#ffffff",
  fontWeight: 600,
  cursor: "pointer",
  transition: "opacity 0.2s ease",
  backgroundColor: "#4f46e5",
};

/**
 * @notice Per-state style overrides.
 * @security All values are hardcoded constants — no dynamic CSS injection from user input.
 */
const STATE_STYLES: Record<SubmitButtonState, React.CSSProperties> = {
const STATE_STYLE_MAP: Record<SubmitButtonState, React.CSSProperties> = {
  idle: { backgroundColor: "#4f46e5" },
  submitting: { backgroundColor: "#6366f1" },
  success: { backgroundColor: "#16a34a", borderColor: "#15803d" },
  error: { backgroundColor: "#dc2626", borderColor: "#b91c1c" },
  disabled: { backgroundColor: "#9ca3af", borderColor: "#9ca3af", cursor: "not-allowed", opacity: 0.9 },
};

// ── Pure helpers (exported for unit testing) ──────────────────────────────────

/**
 * @title normalizeSubmitButtonLabel
 * @notice Sanitizes a candidate label: rejects non-strings, strips control characters,
 *         normalizes whitespace, and truncates to MAX_LABEL_LENGTH.
 * @param candidate  Untrusted input (may be any type).
 * @param fallback   Returned when candidate is unusable.
 * @security Prevents blank CTA states and layout-abuse via oversized labels.
 */
export function normalizeSubmitButtonLabel(candidate: unknown, fallback: string): string {
  if (typeof candidate !== "string") return fallback;
  const cleaned = candidate.replace(CONTROL_CHAR_RE, " ").replace(/\s+/g, " ").trim();
  if (!cleaned) return fallback;
  if (cleaned.length <= MAX_LABEL_LENGTH) return cleaned;
  return `${cleaned.slice(0, MAX_LABEL_LENGTH - 3)}...`;
}

/**
 * @title resolveSubmitButtonLabel
 * @notice Returns a safe, non-empty label for the given state.
 * @param state   Current button state.
 * @param labels  Optional overrides.
 */
export function resolveSubmitButtonLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels,
): string {
  return normalizeSubmitButtonLabel(labels?.[state], DEFAULT_LABELS[state]);
}

/**
 * @title isValidSubmitButtonStateTransition
 * @notice Returns true when moving from `from` to `to` is an allowed transition.
 * @dev Same-state updates are always allowed (idempotent).
 */
export function isValidSubmitButtonStateTransition(
  from: SubmitButtonState,
  to: SubmitButtonState,
): boolean {
  return from === to || ALLOWED_TRANSITIONS[from].includes(to);
}

/**
 * @title resolveSafeSubmitButtonState
 * @notice In strict mode, falls back to `previousState` when the transition is invalid.
 * @param state            Requested next state.
 * @param previousState    Last known valid state.
 * @param strictTransitions  Enables transition enforcement. Default: true.
 */
export function resolveSafeSubmitButtonState(
  state: SubmitButtonState,
  previousState?: SubmitButtonState,
  strictTransitions = true,
): SubmitButtonState {
  if (!strictTransitions || !previousState) return state;
  return isValidSubmitButtonStateTransition(previousState, state) ? state : previousState;
}

/**
 * @title isSubmitButtonInteractionBlocked
 * @notice Returns true when the button must not respond to clicks.
 * @param state              Resolved button state.
 * @param disabled           External disabled flag.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 * @security Prevents duplicate blockchain transactions on rapid clicks.
 */
export function isSubmitButtonInteractionBlocked(
  state: SubmitButtonState,
  disabled = false,
  isLocallySubmitting = false,
): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting" || isLocallySubmitting;
}

/**
 * @title isSubmitButtonBusy
 * @notice Returns true when aria-busy should be set (active submission in progress).
 * @param state              Resolved button state.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 */
export function isSubmitButtonBusy(
  state: SubmitButtonState,
  isLocallySubmitting = false,
): boolean {
  return state === "submitting" || isLocallySubmitting;
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @title ReactSubmitButton
 * @notice Reusable submit button with strict state machine, safe labels,
 *         double-submit prevention, and ARIA accessibility.
 * @dev onClick is wrapped in a local in-flight guard so async handlers cannot
 *      trigger a second submission before the first resolves.
 */
const ReactSubmitButton = ({
  state,
  previousState,
  strictTransitions = true,
/**
 * @title React Submit Button
 * @notice Reusable submit button with typed state machine for scalable workflows.
 * @dev Avoids exposing raw HTML injection paths and enforces accessible semantics.
 */
const ReactSubmitButton = ({
  state,
  labels,
  onClick,
  className,
  id,
  type = "button",
  disabled,
}: ReactSubmitButtonProps) => {
  const [isLocallySubmitting, setIsLocallySubmitting] = useState(false);

  const resolvedState = resolveSafeSubmitButtonState(state, previousState, strictTransitions);
  const label = resolveSubmitButtonLabel(resolvedState, labels);
  const blocked = isSubmitButtonInteractionBlocked(resolvedState, disabled, isLocallySubmitting);
  const ariaBusy = isSubmitButtonBusy(resolvedState, isLocallySubmitting);

  const handleClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    if (blocked || !onClick) return;
    setIsLocallySubmitting(true);
    try {
      await Promise.resolve(onClick(event));
    } catch {
      // Errors are the caller's responsibility; we only reset local state.
    } finally {
      setIsLocallySubmitting(false);
    }
  };
  const label = resolveSubmitButtonLabel(state, labels);
  const computedDisabled = isSubmitButtonDisabled(state, disabled);
  const ariaBusy = isSubmitButtonBusy(state);

  return (
    <button
      id={id}
      type={type}
      className={className}
      disabled={blocked}
      aria-busy={ariaBusy}
      aria-live="polite"
      aria-label={label}
      data-state={resolvedState}
      onClick={blocked ? undefined : handleClick}
      style={{ ...BASE_STYLE, ...STATE_STYLES[resolvedState] }}
      disabled={computedDisabled}
      aria-busy={ariaBusy}
      aria-live="polite"
      aria-label={label}
      onClick={computedDisabled ? undefined : onClick}
      style={{
        ...BASE_STYLE,
        ...STATE_STYLE_MAP[state],
        ...(computedDisabled ? { cursor: "not-allowed", opacity: 0.7 } : {}),
      }}
    >
      {label}
    </button>
  );
};

export default ReactSubmitButton;
