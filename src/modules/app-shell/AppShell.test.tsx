import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AppShell } from "./AppShell";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  TauriEvent: {
    DRAG_DROP: "tauri://drag-drop",
    DRAG_ENTER: "tauri://drag-enter",
    DRAG_LEAVE: "tauri://drag-leave",
  },
  listen: vi.fn(async () => async () => {}),
}));

describe("AppShell", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("renders the empty state shell and loads the ffmpeg path", async () => {
    invokeMock.mockResolvedValue("C:\\bundle\\ffmpeg.exe");

    render(<AppShell />);

    expect(
      screen.getByRole("heading", { name: "Desktop Video Rebranding App" })
    ).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("C:\\bundle\\ffmpeg.exe")).toBeInTheDocument();
    });
  });
});
